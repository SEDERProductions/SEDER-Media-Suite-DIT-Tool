#include "JobQueueModel.h"

JobQueueModel::JobQueueModel(QObject *parent)
    : QAbstractListModel(parent)
{
}

int JobQueueModel::rowCount(const QModelIndex &parent) const
{
    if (parent.isValid()) return 0;
    return m_jobs.size();
}

QVariant JobQueueModel::data(const QModelIndex &index, int role) const
{
    if (!index.isValid() || index.row() < 0 || index.row() >= m_jobs.size())
        return QVariant();

    const Job &j = m_jobs.at(index.row());
    switch (role) {
    case Qt::DisplayRole:
    case LabelRole:
        return j.label;
    case SourcePathRole:
        return j.sourcePath;
    case DestinationCountRole:
        return j.destinationCount;
    case StateRole:
        return j.state;
    case ProgressRole:
        return j.progress;
    case FinalStatusRole:
        return j.finalStatus;
    case TotalFilesRole:
        return static_cast<qulonglong>(j.totalFiles);
    }
    return QVariant();
}

QHash<int, QByteArray> JobQueueModel::roleNames() const
{
    return {
        {LabelRole, "label"},
        {SourcePathRole, "sourcePath"},
        {DestinationCountRole, "destinationCount"},
        {StateRole, "state"},
        {ProgressRole, "progress"},
        {FinalStatusRole, "finalStatus"},
        {TotalFilesRole, "totalFiles"},
    };
}

int JobQueueModel::count() const
{
    return m_jobs.size();
}

int JobQueueModel::activeCount() const
{
    int n = 0;
    for (const Job &j : m_jobs)
        if (j.state == Queued || j.state == Running) ++n;
    return n;
}

int JobQueueModel::enqueue(const OffloadRequestData &request, const QString &label)
{
    beginInsertRows(QModelIndex(), m_jobs.size(), m_jobs.size());
    Job j;
    j.label = label;
    j.sourcePath = request.sourcePath;
    j.destinationCount = request.destinations.size();
    j.state = Queued;
    j.request = request;
    m_jobs.append(j);
    endInsertRows();
    emit countChanged();
    emit activeCountChanged();
    return m_jobs.size() - 1;
}

bool JobQueueModel::hasRequestAt(int index) const
{
    return index >= 0 && index < m_jobs.size();
}

OffloadRequestData JobQueueModel::requestAt(int index) const
{
    if (!hasRequestAt(index)) return {};
    return m_jobs.at(index).request;
}

void JobQueueModel::emitRow(int index, const QVector<int> &roles)
{
    if (index < 0 || index >= m_jobs.size()) return;
    const QModelIndex i = this->index(index, 0);
    emit dataChanged(i, i, roles);
}

void JobQueueModel::setState(int index, int state)
{
    if (index < 0 || index >= m_jobs.size()) return;
    if (m_jobs[index].state == state) return;
    m_jobs[index].state = state;
    emitRow(index, {StateRole});
    emit activeCountChanged();
}

void JobQueueModel::setProgress(int index, double progress)
{
    if (index < 0 || index >= m_jobs.size()) return;
    m_jobs[index].progress = progress;
    emitRow(index, {ProgressRole});
}

void JobQueueModel::setSummary(int index, const QString &finalStatus, quint64 totalFiles)
{
    if (index < 0 || index >= m_jobs.size()) return;
    m_jobs[index].finalStatus = finalStatus;
    m_jobs[index].totalFiles = totalFiles;
    emitRow(index, {FinalStatusRole, TotalFilesRole});
}

int JobQueueModel::nextQueuedIndex() const
{
    for (int i = 0; i < m_jobs.size(); ++i)
        if (m_jobs.at(i).state == Queued) return i;
    return -1;
}

void JobQueueModel::removeJob(int index)
{
    if (index < 0 || index >= m_jobs.size()) return;
    beginRemoveRows(QModelIndex(), index, index);
    m_jobs.removeAt(index);
    endRemoveRows();
    emit countChanged();
    emit activeCountChanged();
}

void JobQueueModel::moveJob(int from, int to)
{
    if (from < 0 || from >= m_jobs.size()) return;
    if (to < 0 || to >= m_jobs.size() || to == from) return;
    // beginMoveRows quirk: destination index is interpreted before removal.
    const int dest = to > from ? to + 1 : to;
    if (!beginMoveRows(QModelIndex(), from, from, QModelIndex(), dest)) return;
    m_jobs.move(from, to);
    endMoveRows();
}

void JobQueueModel::clearFinished()
{
    for (int i = m_jobs.size() - 1; i >= 0; --i) {
        const int s = m_jobs.at(i).state;
        if (s == Complete || s == Failed || s == Cancelled) {
            beginRemoveRows(QModelIndex(), i, i);
            m_jobs.removeAt(i);
            endRemoveRows();
        }
    }
    emit countChanged();
    emit activeCountChanged();
}
