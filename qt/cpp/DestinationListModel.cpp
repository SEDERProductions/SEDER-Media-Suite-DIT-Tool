#include "DestinationListModel.h"

DestinationListModel::DestinationListModel(QObject *parent)
    : QAbstractListModel(parent)
{
}

int DestinationListModel::rowCount(const QModelIndex &parent) const
{
    if (parent.isValid()) return 0;
    return m_items.size();
}

QVariant DestinationListModel::data(const QModelIndex &index, int role) const
{
    if (!index.isValid() || index.row() < 0 || index.row() >= m_items.size())
        return QVariant();

    DestinationItem *item = m_items.at(index.row());
    switch (role) {
    case Qt::DisplayRole:
    case LabelRole:
        return item->label();
    case PathRole:
        return item->path();
    case StateRole:
        return item->state();
    case ProgressRole:
        return item->progress();
    case ErrorRole:
        return item->error();
    }
    return QVariant();
}

QHash<int, QByteArray> DestinationListModel::roleNames() const
{
    QHash<int, QByteArray> roles;
    roles[LabelRole] = "label";
    roles[PathRole] = "path";
    roles[StateRole] = "state";
    roles[ProgressRole] = "progress";
    roles[ErrorRole] = "error";
    return roles;
}

int DestinationListModel::count() const
{
    return m_items.size();
}

void DestinationListModel::addDestination(const QString &path, const QString &label)
{
    beginInsertRows(QModelIndex(), m_items.size(), m_items.size());
    auto *item = new DestinationItem(this);
    item->setPath(path);
    item->setLabel(label.isEmpty() ? QStringLiteral("Destination %1").arg(m_items.size() + 1) : label);
    m_items.append(item);
    endInsertRows();
    emit countChanged();
}

void DestinationListModel::removeDestination(int index)
{
    if (index < 0 || index >= m_items.size()) return;
    beginRemoveRows(QModelIndex(), index, index);
    auto *item = m_items.takeAt(index);
    item->deleteLater();
    endRemoveRows();
    emit countChanged();
}

void DestinationListModel::clear()
{
    if (m_items.isEmpty()) return;
    beginResetModel();
    for (auto *item : m_items) item->deleteLater();
    m_items.clear();
    endResetModel();
    emit countChanged();
}
