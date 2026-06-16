#pragma once

#include <QAbstractListModel>
#include <QString>
#include <QVariantMap>
#include <QVector>

// Browsable list of clips for the Media Library / Inspector. Populated by
// parsing the metadata JSON sidecar the report already emits (see
// src/report.rs `report_metadata_json`); no new FFI is required.
class ClipLibraryModel final : public QAbstractListModel {
    Q_OBJECT
    Q_PROPERTY(int count READ count NOTIFY countChanged)
    Q_PROPERTY(QString sourcePath READ sourcePath NOTIFY sourcePathChanged)

public:
    enum Roles {
        RelPathRole = Qt::UserRole + 1,
        FileNameRole,
        SizeRole,
        SizeTextRole,
        MediaKindRole,
        HasMetadataRole,
        CodecRole,
        ResolutionRole,
        FpsRole,
        DurationRole,
        TimecodeRole,
        AudioRole,
        ColorSpaceRole,
        HashRole,
        AlgorithmRole
    };
    Q_ENUM(Roles)

    explicit ClipLibraryModel(QObject *parent = nullptr);

    int rowCount(const QModelIndex &parent = QModelIndex()) const override;
    QVariant data(const QModelIndex &index, int role = Qt::DisplayRole) const override;
    QHash<int, QByteArray> roleNames() const override;

    int count() const;
    QString sourcePath() const;

    // Replace the model contents from the metadata JSON sidecar. `sourcePath`
    // is the offload source root, used to resolve absolute media paths for
    // thumbnail/proxy generation. Passing empty JSON clears the model.
    void loadFromMetadataJson(const QByteArray &json, const QString &sourcePath);
    void clear();

    // All roles for one row as a map (used by the QML inspector).
    Q_INVOKABLE QVariantMap get(int index) const;

    // Filtered list of row maps (case-insensitive match on file name, codec
    // or path). Empty filter returns every clip. Drives the Library grid +
    // live search without a separate proxy model.
    Q_INVOKABLE QVariantList items(const QString &filter = QString()) const;

    static QString formatBytes(quint64 value);
    static QString formatDuration(double seconds);
    static QString formatFps(double num, double den);

signals:
    void countChanged();
    void sourcePathChanged();

private:
    struct Clip {
        QString relPath;
        QString fileName;
        quint64 size = 0;
        QString sizeText;
        QString mediaKind;
        bool hasMetadata = false;
        QString codec;
        QString resolution;
        QString fps;
        QString duration;
        QString timecode;
        QString audio;
        QString colorSpace;
        QString hash;
        QString algorithm;
    };

    static QVariantMap clipToMap(const Clip &c);

    QVector<Clip> m_clips;
    QString m_sourcePath;
};
