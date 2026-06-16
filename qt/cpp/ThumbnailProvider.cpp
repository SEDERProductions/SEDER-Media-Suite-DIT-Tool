#include "ThumbnailProvider.h"

#include "seder_ffi.h"

#include <QDir>
#include <QImage>
#include <QQuickTextureFactory>
#include <QRunnable>
#include <QStandardPaths>
#include <QThreadPool>
#include <QUrl>

namespace {

class ThumbnailResponse final : public QQuickImageResponse, public QRunnable {
public:
    ThumbnailResponse(const QString &id, const QSize &requestedSize)
        : m_requestedSize(requestedSize)
    {
        // id = "<algo>/<hash>/<percent-encoded abs media path>". Parse from the
        // left so the (possibly slash-containing) media path is the remainder;
        // fromPercentEncoding is a no-op if Qt already decoded the id.
        const int firstSlash = id.indexOf(u'/');
        const int secondSlash = firstSlash >= 0 ? id.indexOf(u'/', firstSlash + 1) : -1;
        if (firstSlash >= 0 && secondSlash >= 0) {
            m_algo = id.left(firstSlash);
            m_hash = id.mid(firstSlash + 1, secondSlash - firstSlash - 1);
            m_media = QUrl::fromPercentEncoding(id.mid(secondSlash + 1).toUtf8());
        }
        setAutoDelete(false);
        QThreadPool::globalInstance()->start(this);
    }

    void run() override
    {
        if (!m_media.isEmpty() && !m_hash.isEmpty() && !m_algo.isEmpty()) {
            const QString cacheDir =
                QDir(QStandardPaths::writableLocation(QStandardPaths::CacheLocation))
                    .filePath(QStringLiteral("thumbnails"));
            QDir().mkpath(cacheDir);

            const QByteArray mediaB = m_media.toUtf8();
            const QByteArray cacheB = cacheDir.toUtf8();
            const QByteArray algoB = m_algo.toUtf8();
            const QByteArray hashB = m_hash.toUtf8();
            char *path = seder_extract_thumbnail(mediaB.constData(), cacheB.constData(),
                                                 algoB.constData(), hashB.constData());
            if (path) {
                const QString jpeg = QString::fromUtf8(path);
                seder_string_free(path);
                QImage img(jpeg);
                if (!img.isNull()) {
                    if (m_requestedSize.isValid() && !m_requestedSize.isEmpty())
                        img = img.scaled(m_requestedSize, Qt::KeepAspectRatio, Qt::SmoothTransformation);
                    m_image = img;
                }
            }
        }
        if (m_image.isNull())
            m_error = QStringLiteral("thumbnail unavailable");
        emit finished();
    }

    QQuickTextureFactory *textureFactory() const override
    {
        return QQuickTextureFactory::textureFactoryForImage(m_image);
    }

    QString errorString() const override { return m_error; }

private:
    QSize m_requestedSize;
    QString m_algo;
    QString m_hash;
    QString m_media;
    QString m_error;
    QImage m_image;
};

} // namespace

QQuickImageResponse *ThumbnailProvider::requestImageResponse(const QString &id, const QSize &requestedSize)
{
    return new ThumbnailResponse(id, requestedSize);
}
