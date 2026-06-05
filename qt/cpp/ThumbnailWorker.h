#pragma once

#include <QAtomicInt>
#include <QObject>
#include <QString>
#include <QVector>

// Generates clip thumbnails on a background thread by calling the Rust core
// (which shells out to ffmpeg). Mirrors DitOffloadWorker: a QObject moved
// onto a QThread, emitting one result per job back to the UI thread via a
// queued connection, with cooperative cancellation when the source changes.
class ThumbnailWorker : public QObject {
    Q_OBJECT
public:
    struct Job {
        QString absolutePath;
        // Synthetic cache key (size + mtime) — NOT a content hash. The Rust
        // side only uses it as the cache filename, so a cheap key lets us
        // preview a card without reading every byte.
        QString cacheKey;
    };

    ThumbnailWorker(QVector<Job> jobs, QString cacheDir, QObject *parent = nullptr);

    void cancel();

public slots:
    void run();

signals:
    // ok == false (and an empty thumbPath) when ffmpeg can't produce a frame.
    void thumbnailReady(QString absolutePath, QString thumbPath, bool ok);
    void finished();

private:
    QVector<Job> m_jobs;
    QString m_cacheDir;
    QAtomicInt m_cancel{0};
};
