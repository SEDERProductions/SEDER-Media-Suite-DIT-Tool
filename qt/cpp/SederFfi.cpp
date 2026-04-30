#include "SederFfi.h"

#include "seder_ffi.h"

#include <QByteArray>
#include <stdexcept>
#include <utility>

namespace {

struct ProgressContext {
    std::function<void(const DitProgress &)> callback;
};

QString borrowedString(const char *value)
{
    return value ? QString::fromUtf8(value) : QString();
}

QString takeOwnedString(char *value)
{
    if (!value) {
        return {};
    }
    const QString result = QString::fromUtf8(value);
    seder_string_free(value);
    return result;
}

void progressThunk(
    const char *phase,
    uint64_t processedFiles,
    uint64_t processedBytes,
    const char *status,
    void *userData)
{
    auto *context = static_cast<ProgressContext *>(userData);
    if (!context || !context->callback) {
        return;
    }
    context->callback(DitProgress {
        borrowedString(phase),
        static_cast<quint64>(processedFiles),
        static_cast<quint64>(processedBytes),
        borrowedString(status),
    });
}

quint64 optionalSize(const SederDitReportHandle *handle, uint64_t row, bool left, bool *ok)
{
    uint64_t value = 0;
    const uint8_t present = left
        ? seder_dit_report_row_size_a(handle, row, &value)
        : seder_dit_report_row_size_b(handle, row, &value);
    *ok = present != 0;
    return static_cast<quint64>(value);
}

} // namespace

DitResult SederFfi::compare(
    const DitRequestData &request,
    const std::function<void(const DitProgress &)> &progress)
{
    const QByteArray sourcePath = request.sourcePath.toUtf8();
    const QByteArray destinationPath = request.destinationPath.toUtf8();
    const QByteArray projectName = request.projectName.toUtf8();
    const QByteArray shootDate = request.shootDate.toUtf8();
    const QByteArray cardName = request.cardName.toUtf8();
    const QByteArray cameraId = request.cameraId.toUtf8();
    const QByteArray ignorePatterns = request.ignorePatterns.toUtf8();

    SederDitRequest ffiRequest {
        sourcePath.constData(),
        destinationPath.constData(),
        projectName.constData(),
        shootDate.constData(),
        cardName.constData(),
        cameraId.constData(),
        ignorePatterns.constData(),
        static_cast<uint32_t>(request.compareMode),
        static_cast<uint8_t>(request.ignoreHiddenSystem ? 1 : 0),
    };

    ProgressContext context { progress };
    char *error = nullptr;
    SederDitReportHandle *handle = seder_dit_compare(&ffiRequest, progressThunk, &context, &error);
    if (!handle) {
        const QString message = takeOwnedString(error);
        throw std::runtime_error((message.isEmpty() ? QStringLiteral("DIT comparison failed") : message).toStdString());
    }

    DitResult result;
    const uint64_t rowCount = seder_dit_report_row_count(handle);
    result.rows.reserve(static_cast<qsizetype>(rowCount));
    for (uint64_t i = 0; i < rowCount; ++i) {
        DitResultRow row;
        row.status = seder_dit_report_row_status(handle, i);
        row.relativePath = borrowedString(seder_dit_report_row_path(handle, i));
        row.folder = seder_dit_report_row_is_folder(handle, i) != 0;
        row.sizeA = optionalSize(handle, i, true, &row.hasSizeA);
        row.sizeB = optionalSize(handle, i, false, &row.hasSizeB);
        row.checksumA = borrowedString(seder_dit_report_row_checksum_a(handle, i));
        row.checksumB = borrowedString(seder_dit_report_row_checksum_b(handle, i));
        result.rows.push_back(std::move(row));
    }

    SederDitSummary summary {};
    if (seder_dit_report_summary(handle, &summary) != 0) {
        result.summary.onlyA = static_cast<quint64>(summary.only_a);
        result.summary.onlyB = static_cast<quint64>(summary.only_b);
        result.summary.changed = static_cast<quint64>(summary.changed);
        result.summary.matching = static_cast<quint64>(summary.matching);
        result.summary.totalFiles = static_cast<quint64>(summary.total_files);
        result.summary.totalFolders = static_cast<quint64>(summary.total_folders);
        result.summary.totalSize = static_cast<quint64>(summary.total_size);
        result.summary.pass = summary.pass != 0;
        result.summary.mhlAvailable = summary.mhl_available != 0;
        result.summary.compareMode = static_cast<int>(summary.compare_mode);
    }

    result.txtExport = takeOwnedString(seder_dit_report_export_txt(handle));
    result.csvExport = takeOwnedString(seder_dit_report_export_csv(handle));
    if (result.summary.mhlAvailable) {
        result.mhlExport = takeOwnedString(seder_dit_report_export_mhl(handle));
    }

    seder_dit_report_free(handle);
    return result;
}
