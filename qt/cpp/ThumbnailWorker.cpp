#include "ThumbnailWorker.h"
#include "seder_ffi.h"

#include <QByteArray>

ThumbnailWorker::ThumbnailWorker(QVector<Job> jobs, QString cacheDir, QObject *parent)
    : QObject(parent)
    , m_jobs(std::move(jobs))
    , m_cacheDir(std::move(cacheDir))
{
}

void ThumbnailWorker::cancel()
{
    m_cancel.storeRelaxed(1);
}

void ThumbnailWorker::run()
{
    const QByteArray cacheDir = m_cacheDir.toUtf8();
    // Cache namespace for browser previews; keeps them separate from any
    // future content-addressed (post-offload) thumbnails.
    const QByteArray algorithm = QByteArrayLiteral("PREVIEW");

    for (const Job &job : m_jobs) {
        if (m_cancel.loadRelaxed() != 0) break;

        // Keep the UTF-8 buffers alive across the FFI call, then free the
        // heap string the Rust side hands back (same discipline as
        // DitOffloadWorker / previewDestinationTemplate).
        const QByteArray media = job.absolutePath.toUtf8();
        const QByteArray key = job.cacheKey.toUtf8();

        char *out = seder_extract_thumbnail(
            media.constData(),
            cacheDir.constData(),
            algorithm.constData(),
            key.constData());

        if (m_cancel.loadRelaxed() != 0) {
            if (out) seder_string_free(out);
            break;
        }

        if (out) {
            const QString path = QString::fromUtf8(out);
            seder_string_free(out);
            emit thumbnailReady(job.absolutePath, path, true);
        } else {
            emit thumbnailReady(job.absolutePath, QString(), false);
        }
    }

    emit finished();
}
