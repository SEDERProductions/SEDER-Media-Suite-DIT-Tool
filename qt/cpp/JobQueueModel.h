#pragma once

#include "DitOffloadWorker.h" // OffloadRequestData

#include <QAbstractListModel>
#include <QVector>

// ShotPut-style queue of offload jobs. Each row carries the immutable job
// request plus live state/progress. AppController runs jobs serially.
class JobQueueModel final : public QAbstractListModel {
    Q_OBJECT
    Q_PROPERTY(int count READ count NOTIFY countChanged)
    Q_PROPERTY(int activeCount READ activeCount NOTIFY activeCountChanged)

public:
    enum JobState { Queued = 0, Running = 1, Complete = 2, Failed = 3, Cancelled = 4 };
    Q_ENUM(JobState)

    enum Roles {
        LabelRole = Qt::UserRole + 1,
        SourcePathRole,
        DestinationCountRole,
        StateRole,
        ProgressRole,
        FinalStatusRole,
        TotalFilesRole
    };
    Q_ENUM(Roles)

    explicit JobQueueModel(QObject *parent = nullptr);

    int rowCount(const QModelIndex &parent = QModelIndex()) const override;
    QVariant data(const QModelIndex &index, int role = Qt::DisplayRole) const override;
    QHash<int, QByteArray> roleNames() const override;

    int count() const;
    // Number of jobs not yet finished (Queued or Running).
    int activeCount() const;

    int enqueue(const OffloadRequestData &request, const QString &label);
    bool hasRequestAt(int index) const;
    OffloadRequestData requestAt(int index) const;

    void setState(int index, int state);
    void setProgress(int index, double progress);
    void setSummary(int index, const QString &finalStatus, quint64 totalFiles);

    // First job still in the Queued state, or -1.
    int nextQueuedIndex() const;

    Q_INVOKABLE void removeJob(int index);
    Q_INVOKABLE void moveJob(int from, int to);
    Q_INVOKABLE void clearFinished();

signals:
    void countChanged();
    void activeCountChanged();

private:
    struct Job {
        QString label;
        QString sourcePath;
        int destinationCount = 0;
        int state = Queued;
        double progress = 0.0;
        QString finalStatus;
        quint64 totalFiles = 0;
        OffloadRequestData request;
    };

    void emitRow(int index, const QVector<int> &roles);

    QVector<Job> m_jobs;
};
