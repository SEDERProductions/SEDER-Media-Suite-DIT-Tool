#include "DitFilterProxyModel.h"

#include "DitResultModel.h"
#include "seder_ffi.h"

DitFilterProxyModel::DitFilterProxyModel(QObject *parent)
    : QSortFilterProxyModel(parent)
{
    setDynamicSortFilter(true);
    connect(this, &QAbstractItemModel::modelReset, this, &DitFilterProxyModel::rowCountChanged);
    connect(this, &QAbstractItemModel::rowsInserted, this, &DitFilterProxyModel::rowCountChanged);
    connect(this, &QAbstractItemModel::rowsRemoved, this, &DitFilterProxyModel::rowCountChanged);
}

int DitFilterProxyModel::filter() const
{
    return m_filter;
}

void DitFilterProxyModel::setFilter(int filter)
{
    if (m_filter == filter) {
        return;
    }
    m_filter = filter;
    invalidateFilter();
    emit filterChanged();
    emit rowCountChanged();
}

int DitFilterProxyModel::visibleRowCount() const
{
    return rowCount();
}

bool DitFilterProxyModel::filterAcceptsRow(int sourceRow, const QModelIndex &sourceParent) const
{
    if (m_filter == All) {
        return true;
    }
    const QModelIndex index = sourceModel()->index(sourceRow, 0, sourceParent);
    const int status = sourceModel()->data(index, DitResultModel::StatusRole).toInt();
    const bool folder = sourceModel()->data(index, DitResultModel::IsFolderRole).toBool();

    switch (m_filter) {
    case Matching: return status == SEDER_ROW_MATCHING;
    case Changed: return status == SEDER_ROW_CHANGED;
    case OnlyA: return status == SEDER_ROW_ONLY_IN_A || status == SEDER_ROW_FOLDER_ONLY_IN_A;
    case OnlyB: return status == SEDER_ROW_ONLY_IN_B || status == SEDER_ROW_FOLDER_ONLY_IN_B;
    case Folders: return folder;
    default: return true;
    }
}
