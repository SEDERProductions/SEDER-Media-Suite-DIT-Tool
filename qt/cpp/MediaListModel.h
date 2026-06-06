#pragma once

#include <QAbstractListModel>
#include <QHash>
#include <QString>
#include <QUrl>
#include <QVector>

// Read-mostly list of the media files discovered in the source folder,
// populated from the JSON that seder_scan_media_list() returns. Each row
// carries a thumbnail URL + state that the background ThumbnailWorker fills
// in asynchronously. Backed by a plain-struct vector (no per-row QObject):
// the grid only ever reads through model roles, and thumbnail updates go
// through dataChanged().
class MediaListModel : public QAbstractListModel {
    Q_OBJECT
    Q_PROPERTY(int count READ count NOTIFY countChanged)

public:
    // Thumbnail lifecycle for a row, mirrored by the QML delegate.
    enum ThumbState {
        NoThumb = 0,     // video candidate, not yet requested
        Pending = 1,     // generation in flight
        Ready = 2,       // thumbnailUrl is valid
        Failed = 3,      // ffmpeg failed or unavailable -> show badge
        Unsupported = 4  // non-video kind: badge only, never spawn ffmpeg
    };
    Q_ENUM(ThumbState)

    enum Roles {
        FileNameRole = Qt::UserRole + 1,
        RelativePathRole,
        AbsolutePathRole,
        SizeRole,
        KindRole,
        ThumbnailUrlRole,
        ThumbnailStateRole
    };
    Q_ENUM(Roles)

    struct Entry {
        QString fileName;
        QString relativePath;
        QString absolutePath;
        quint64 size = 0;
        QString kind;
        QUrl thumbnailUrl;
        int thumbnailState = NoThumb;
    };

    explicit MediaListModel(QObject *parent = nullptr);

    int rowCount(const QModelIndex &parent = QModelIndex()) const override;
    QVariant data(const QModelIndex &index, int role = Qt::DisplayRole) const override;
    QHash<int, QByteArray> roleNames() const override;

    int count() const;

    // Rebuild every row from the JSON array produced by seder_scan_media_list.
    void resetFromJson(const QByteArray &json);
    void clear();

    // Snapshot used by AppController to enqueue thumbnail jobs.
    QVector<Entry> entries() const { return m_entries; }

    // Update a row located by absolutePath. Both are no-ops when the path is
    // no longer present (e.g. a stale result after the source changed), which
    // is what makes cross-generation thumbnail results safe to ignore.
    void setThumbnailState(const QString &absolutePath, int state);
    void setThumbnail(const QString &absolutePath, const QUrl &url, int state);

signals:
    void countChanged();

private:
    static bool isThumbnailableKind(const QString &kind);

    QVector<Entry> m_entries;
    QHash<QString, int> m_rowByPath; // absolutePath -> row index
};
