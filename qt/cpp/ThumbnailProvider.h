#pragma once

#include <QQuickAsyncImageProvider>

// Async, disk-cached clip thumbnails. QML requests:
//   image://clipthumb/<algorithm>/<hash>/<percent-encoded absolute media path>
// The work is delegated to seder_extract_thumbnail (itself content-addressed
// and disk-cached) on a thread pool so the UI never blocks on ffmpeg.
class ThumbnailProvider final : public QQuickAsyncImageProvider {
public:
    QQuickImageResponse *requestImageResponse(const QString &id, const QSize &requestedSize) override;
};
