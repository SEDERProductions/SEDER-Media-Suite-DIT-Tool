#include "ClipLibraryModel.h"

#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QJsonValue>
#include <cmath>

ClipLibraryModel::ClipLibraryModel(QObject *parent)
    : QAbstractListModel(parent)
{
}

int ClipLibraryModel::rowCount(const QModelIndex &parent) const
{
    if (parent.isValid()) return 0;
    return m_clips.size();
}

QVariant ClipLibraryModel::data(const QModelIndex &index, int role) const
{
    if (!index.isValid() || index.row() < 0 || index.row() >= m_clips.size())
        return QVariant();

    const Clip &c = m_clips.at(index.row());
    switch (role) {
    case Qt::DisplayRole:
    case FileNameRole:
        return c.fileName;
    case RelPathRole:
        return c.relPath;
    case SizeRole:
        return static_cast<qulonglong>(c.size);
    case SizeTextRole:
        return c.sizeText;
    case MediaKindRole:
        return c.mediaKind;
    case HasMetadataRole:
        return c.hasMetadata;
    case CodecRole:
        return c.codec;
    case ResolutionRole:
        return c.resolution;
    case FpsRole:
        return c.fps;
    case DurationRole:
        return c.duration;
    case TimecodeRole:
        return c.timecode;
    case AudioRole:
        return c.audio;
    case ColorSpaceRole:
        return c.colorSpace;
    case HashRole:
        return c.hash;
    case AlgorithmRole:
        return c.algorithm;
    }
    return QVariant();
}

QHash<int, QByteArray> ClipLibraryModel::roleNames() const
{
    return {
        {RelPathRole, "relPath"},
        {FileNameRole, "fileName"},
        {SizeRole, "size"},
        {SizeTextRole, "sizeText"},
        {MediaKindRole, "mediaKind"},
        {HasMetadataRole, "hasMetadata"},
        {CodecRole, "codec"},
        {ResolutionRole, "resolution"},
        {FpsRole, "fps"},
        {DurationRole, "duration"},
        {TimecodeRole, "timecode"},
        {AudioRole, "audio"},
        {ColorSpaceRole, "colorSpace"},
        {HashRole, "hash"},
        {AlgorithmRole, "algorithm"},
    };
}

int ClipLibraryModel::count() const
{
    return m_clips.size();
}

QString ClipLibraryModel::sourcePath() const
{
    return m_sourcePath;
}

void ClipLibraryModel::clear()
{
    if (m_clips.isEmpty() && m_sourcePath.isEmpty()) return;
    beginResetModel();
    m_clips.clear();
    endResetModel();
    if (!m_sourcePath.isEmpty()) {
        m_sourcePath.clear();
        emit sourcePathChanged();
    }
    emit countChanged();
}

void ClipLibraryModel::loadFromMetadataJson(const QByteArray &json, const QString &sourcePath)
{
    QVector<Clip> next;

    if (!json.trimmed().isEmpty()) {
        QJsonParseError err;
        const QJsonDocument doc = QJsonDocument::fromJson(json, &err);
        if (err.error == QJsonParseError::NoError && doc.isObject()) {
            const QJsonArray files = doc.object().value(QStringLiteral("files")).toArray();
            next.reserve(files.size());
            for (const QJsonValue &v : files) {
                const QJsonObject o = v.toObject();
                Clip c;
                c.relPath = o.value(QStringLiteral("path")).toString();
                const int slash = c.relPath.lastIndexOf(u'/');
                c.fileName = slash >= 0 ? c.relPath.mid(slash + 1) : c.relPath;
                c.size = static_cast<quint64>(o.value(QStringLiteral("size")).toVariant().toULongLong());
                c.sizeText = formatBytes(c.size);
                c.mediaKind = o.value(QStringLiteral("media_kind")).toString();
                c.hash = o.value(QStringLiteral("hash")).toString();
                c.algorithm = o.value(QStringLiteral("hash_algorithm")).toString();

                const QJsonValue meta = o.value(QStringLiteral("metadata"));
                if (meta.isObject()) {
                    const QJsonObject m = meta.toObject();
                    c.hasMetadata = true;
                    const QString videoCodec = m.value(QStringLiteral("video_codec")).toString();
                    const QString audioCodec = m.value(QStringLiteral("audio_codec")).toString();
                    c.codec = !videoCodec.isEmpty() ? videoCodec : audioCodec;

                    const int w = m.value(QStringLiteral("width")).toInt();
                    const int h = m.value(QStringLiteral("height")).toInt();
                    if (w > 0 && h > 0)
                        c.resolution = QStringLiteral("%1 × %2").arg(w).arg(h);

                    c.fps = formatFps(m.value(QStringLiteral("fps_num")).toDouble(),
                                      m.value(QStringLiteral("fps_den")).toDouble());
                    c.duration = formatDuration(m.value(QStringLiteral("duration_seconds")).toDouble());
                    c.timecode = m.value(QStringLiteral("timecode")).toString();
                    c.colorSpace = m.value(QStringLiteral("color_space")).toString();

                    if (!audioCodec.isEmpty()) {
                        QStringList parts{audioCodec};
                        const int ch = m.value(QStringLiteral("audio_channels")).toInt();
                        const int sr = m.value(QStringLiteral("audio_sample_rate")).toInt();
                        if (ch > 0) parts << QStringLiteral("%1 ch").arg(ch);
                        if (sr > 0) parts << QStringLiteral("%1 kHz").arg(sr / 1000.0, 0, 'g', 4);
                        c.audio = parts.join(QStringLiteral(" · "));
                    }
                }
                next.append(std::move(c));
            }
        }
    }

    beginResetModel();
    m_clips = std::move(next);
    endResetModel();
    emit countChanged();

    if (m_sourcePath != sourcePath) {
        m_sourcePath = sourcePath;
        emit sourcePathChanged();
    }
}

QVariantMap ClipLibraryModel::clipToMap(const Clip &c)
{
    QVariantMap map;
    map.insert(QStringLiteral("relPath"), c.relPath);
    map.insert(QStringLiteral("fileName"), c.fileName);
    map.insert(QStringLiteral("size"), static_cast<qulonglong>(c.size));
    map.insert(QStringLiteral("sizeText"), c.sizeText);
    map.insert(QStringLiteral("mediaKind"), c.mediaKind);
    map.insert(QStringLiteral("hasMetadata"), c.hasMetadata);
    map.insert(QStringLiteral("codec"), c.codec);
    map.insert(QStringLiteral("resolution"), c.resolution);
    map.insert(QStringLiteral("fps"), c.fps);
    map.insert(QStringLiteral("duration"), c.duration);
    map.insert(QStringLiteral("timecode"), c.timecode);
    map.insert(QStringLiteral("audio"), c.audio);
    map.insert(QStringLiteral("colorSpace"), c.colorSpace);
    map.insert(QStringLiteral("hash"), c.hash);
    map.insert(QStringLiteral("algorithm"), c.algorithm);
    return map;
}

QVariantMap ClipLibraryModel::get(int index) const
{
    if (index < 0 || index >= m_clips.size()) return {};
    return clipToMap(m_clips.at(index));
}

QVariantList ClipLibraryModel::items(const QString &filter) const
{
    QVariantList list;
    const QString needle = filter.trimmed();
    for (const Clip &c : m_clips) {
        if (!needle.isEmpty()
            && !c.fileName.contains(needle, Qt::CaseInsensitive)
            && !c.codec.contains(needle, Qt::CaseInsensitive)
            && !c.relPath.contains(needle, Qt::CaseInsensitive)) {
            continue;
        }
        list.append(clipToMap(c));
    }
    return list;
}

QString ClipLibraryModel::formatBytes(quint64 value)
{
    static const char *units[] = {"B", "KB", "MB", "GB", "TB"};
    if (value == 0) return QStringLiteral("0 B");
    int exp = static_cast<int>(std::min<double>(std::log(static_cast<double>(value)) / std::log(1024.0), 4.0));
    const double scaled = static_cast<double>(value) / std::pow(1024.0, exp);
    if (exp == 0)
        return QStringLiteral("%1 B").arg(value);
    return QStringLiteral("%1 %2").arg(scaled, 0, 'f', 2).arg(QString::fromLatin1(units[exp]));
}

QString ClipLibraryModel::formatDuration(double seconds)
{
    if (!(seconds > 0.0)) return QString();
    const qint64 total = static_cast<qint64>(std::llround(seconds));
    const qint64 h = total / 3600;
    const qint64 m = (total / 60) % 60;
    const qint64 s = total % 60;
    return QStringLiteral("%1:%2:%3")
        .arg(h, 2, 10, QLatin1Char('0'))
        .arg(m, 2, 10, QLatin1Char('0'))
        .arg(s, 2, 10, QLatin1Char('0'));
}

QString ClipLibraryModel::formatFps(double num, double den)
{
    if (den <= 0.0 || num <= 0.0) return QString();
    const double fps = num / den;
    if (std::abs(fps - std::round(fps)) < 0.001)
        return QStringLiteral("%1 fps").arg(fps, 0, 'f', 0);
    return QStringLiteral("%1 fps").arg(fps, 0, 'f', 3);
}
