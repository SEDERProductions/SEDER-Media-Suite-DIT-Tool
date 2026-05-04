#include "DitOffloadWorker.h"
#include "seder_ffi.h"

#include <QByteArray>
#include <QThread>

DitOffloadWorker::DitOffloadWorker(OffloadRequestData request, QObject *parent)
    : QObject(parent)
    , m_request(std::move(request))
{
}

void DitOffloadWorker::cancel()
{
    m_cancelToken.storeRelaxed(1);
}

static void progressCallback(const SederOffloadProgress *progress, void *userData)
{
    auto *worker = static_cast<DitOffloadWorker *>(userData);
    if (!progress) return;

    OffloadProgressData data;
    data.phase = QString::fromUtf8(progress->phase);
    data.overallFilesCompleted = progress->overall_files_completed;
    data.overallFilesTotal = progress->overall_files_total;
    data.overallBytesCompleted = progress->overall_bytes_completed;
    data.overallBytesTotal = progress->overall_bytes_total;
    data.currentFile = QString::fromUtf8(progress->current_file);

    for (size_t i = 0; i < progress->destination_count; ++i) {
        const SederDestinationProgress &dp = progress->destinations[i];
        DestinationProgressData dpd;
        dpd.state = dp.state;
        dpd.filesCompleted = dp.files_completed;
        dpd.filesTotal = dp.files_total;
        dpd.bytesCompleted = dp.bytes_completed;
        dpd.bytesTotal = dp.bytes_total;
        dpd.currentFile = QString::fromUtf8(dp.current_file);
        dpd.error = QString::fromUtf8(dp.error);
        data.destinations.append(dpd);
    }

    emit worker->progress(data);
}

void DitOffloadWorker::run()
{
    QByteArray sourcePath = m_request.sourcePath.toUtf8();

    QVector<QByteArray> destPaths;
    QVector<QByteArray> destLabels;
    QVector<SederDestinationConfig> destConfigs;
    for (const auto &dest : m_request.destinations) {
        destPaths.append(dest.path.toUtf8());
        destLabels.append(dest.label.toUtf8());
        SederDestinationConfig cfg{};
        cfg.path = destPaths.last().constData();
        cfg.label = destLabels.last().isEmpty() ? nullptr : destLabels.last().constData();
        destConfigs.append(cfg);
    }

    QByteArray projectName = m_request.projectName.toUtf8();
    QByteArray shootDate = m_request.shootDate.toUtf8();
    QByteArray cardName = m_request.cardName.toUtf8();
    QByteArray cameraId = m_request.cameraId.toUtf8();
    QByteArray ignorePatterns = m_request.ignorePatterns.toUtf8();

    SederOffloadRequest req{};
    req.source_path = sourcePath.constData();
    req.destinations = destConfigs.data();
    req.destination_count = static_cast<size_t>(destConfigs.size());
    req.project_name = projectName.constData();
    req.shoot_date = shootDate.constData();
    req.card_name = cardName.constData();
    req.camera_id = cameraId.constData();
    req.ignore_patterns = ignorePatterns.constData();
    req.ignore_hidden_system = m_request.ignoreHiddenSystem ? 1 : 0;
    req.verify_after_copy = m_request.verifyAfterCopy ? 1 : 0;
    req.cancel_token = reinterpret_cast<volatile uint8_t *>(&m_cancelToken);

    char *errorOut = nullptr;
    OffloadReportHandle *handle = seder_offload_start(&req, progressCallback, this, &errorOut);

    if (handle) {
        seder_report_free(handle);
    }

    if (m_cancelToken.loadRelaxed() != 0) {
        emit cancelled();
        if (errorOut) seder_string_free(errorOut);
        return;
    }

    if (!handle && errorOut) {
        QString msg = QString::fromUtf8(errorOut);
        seder_string_free(errorOut);
        emit failed(msg);
        return;
    }

    if (!handle) {
        emit failed(QStringLiteral("Unknown offload error"));
        return;
    }

    emit finished();
}
