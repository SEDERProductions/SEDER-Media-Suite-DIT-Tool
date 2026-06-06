#include "MediaListModel.h"

#include <QFileInfo>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>

MediaListModel::MediaListModel(QObject *parent)
    : QAbstractListModel(parent)
{
}

int MediaListModel::rowCount(const QModelIndex &parent) const
{
    if (parent.isValid()) return 0;
    return m_entries.size();
}

QVariant MediaListModel::data(const QModelIndex &index, int role) const
{
    if (!index.isValid() || index.row() < 0 || index.row() >= m_entries.size()) {
        return {};
    }
    const Entry &e = m_entries.at(index.row());
    switch (role) {
    case Qt::DisplayRole:
    case FileNameRole: return e.fileName;
    case RelativePathRole: return e.relativePath;
    case AbsolutePathRole: return e.absolutePath;
    case SizeRole: return QVariant::fromValue(e.size);
    case KindRole: return e.kind;
    case ThumbnailUrlRole: return e.thumbnailUrl;
    case ThumbnailStateRole: return e.thumbnailState;
    default: return {};
    }
}

QHash<int, QByteArray> MediaListModel::roleNames() const
{
    return {
        { FileNameRole, "fileName" },
        { RelativePathRole, "relativePath" },
        { AbsolutePathRole, "absolutePath" },
        { SizeRole, "size" },
        { KindRole, "kind" },
        { ThumbnailUrlRole, "thumbnailUrl" },
        { ThumbnailStateRole, "thumbnailState" },
    };
}

int MediaListModel::count() const
{
    return m_entries.size();
}

bool MediaListModel::isThumbnailableKind(const QString &kind)
{
    // Non-visual kinds (audio, captions, sidecars, unknown) get a format
    // badge immediately and never spawn ffmpeg. Everything else is a video
    // candidate; if ffmpeg can't decode it, the row falls back to a badge.
    return kind != QLatin1String("Audio")
        && kind != QLatin1String("Subtitle")
        && kind != QLatin1String("Sidecar")
        && kind != QLatin1String("Other");
}

void MediaListModel::resetFromJson(const QByteArray &json)
{
    beginResetModel();
    m_entries.clear();
    m_rowByPath.clear();

    const QJsonDocument doc = QJsonDocument::fromJson(json);
    if (doc.isArray()) {
        const QJsonArray arr = doc.array();
        m_entries.reserve(arr.size());
        for (const QJsonValue &v : arr) {
            const QJsonObject o = v.toObject();
            Entry e;
            e.relativePath = o.value(QStringLiteral("rel_path")).toString();
            e.absolutePath = o.value(QStringLiteral("abs_path")).toString();
            e.size = static_cast<quint64>(o.value(QStringLiteral("size")).toDouble());
            e.kind = o.value(QStringLiteral("kind")).toString();
            e.fileName = e.relativePath.section(QLatin1Char('/'), -1);
            e.thumbnailState = isThumbnailableKind(e.kind) ? NoThumb : Unsupported;
            m_rowByPath.insert(e.absolutePath, m_entries.size());
            m_entries.append(e);
        }
    }

    endResetModel();
    emit countChanged();
}

void MediaListModel::clear()
{
    if (m_entries.isEmpty()) return;
    beginResetModel();
    m_entries.clear();
    m_rowByPath.clear();
    endResetModel();
    emit countChanged();
}

void MediaListModel::setThumbnailState(const QString &absolutePath, int state)
{
    const auto it = m_rowByPath.constFind(absolutePath);
    if (it == m_rowByPath.constEnd()) return;
    const int row = it.value();
    if (row < 0 || row >= m_entries.size()) return;
    if (m_entries[row].thumbnailState == state) return;
    m_entries[row].thumbnailState = state;
    const QModelIndex idx = index(row);
    emit dataChanged(idx, idx, { ThumbnailStateRole });
}

void MediaListModel::setThumbnail(const QString &absolutePath, const QUrl &url, int state)
{
    const auto it = m_rowByPath.constFind(absolutePath);
    if (it == m_rowByPath.constEnd()) return;
    const int row = it.value();
    if (row < 0 || row >= m_entries.size()) return;
    m_entries[row].thumbnailUrl = url;
    m_entries[row].thumbnailState = state;
    const QModelIndex idx = index(row);
    emit dataChanged(idx, idx, { ThumbnailUrlRole, ThumbnailStateRole });
}
