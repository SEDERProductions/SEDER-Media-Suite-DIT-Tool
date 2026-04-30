#include "DitResultModel.h"

#include "seder_ffi.h"

#include <utility>

DitResultModel::DitResultModel(QObject *parent)
    : QAbstractTableModel(parent)
{
}

int DitResultModel::rowCount(const QModelIndex &parent) const
{
    return parent.isValid() ? 0 : m_rows.size();
}

int DitResultModel::columnCount(const QModelIndex &parent) const
{
    return parent.isValid() ? 0 : 6;
}

QVariant DitResultModel::data(const QModelIndex &index, int role) const
{
    if (!index.isValid() || index.row() < 0 || index.row() >= m_rows.size()) {
        return {};
    }

    const auto &row = m_rows.at(index.row());
    if (role == Qt::DisplayRole) {
        switch (index.column()) {
        case 0: return statusText(row.status);
        case 1: return row.relativePath;
        case 2: return row.hasSizeA ? formatBytes(row.sizeA) : QString();
        case 3: return row.hasSizeB ? formatBytes(row.sizeB) : QString();
        case 4: return row.checksumA;
        case 5: return row.checksumB;
        default: return {};
        }
    }

    switch (role) {
    case StatusRole: return row.status;
    case StatusTextRole: return statusText(row.status);
    case RelativePathRole: return row.relativePath;
    case SizeARole: return row.hasSizeA ? QVariant::fromValue(row.sizeA) : QVariant();
    case SizeBRole: return row.hasSizeB ? QVariant::fromValue(row.sizeB) : QVariant();
    case ChecksumARole: return row.checksumA;
    case ChecksumBRole: return row.checksumB;
    case IsFolderRole: return row.folder;
    default: return {};
    }
}

QVariant DitResultModel::headerData(int section, Qt::Orientation orientation, int role) const
{
    if (orientation != Qt::Horizontal || role != Qt::DisplayRole) {
        return {};
    }
    switch (section) {
    case 0: return tr("Status");
    case 1: return tr("Relative Path");
    case 2: return tr("Size A");
    case 3: return tr("Size B");
    case 4: return tr("Checksum A");
    case 5: return tr("Checksum B");
    default: return {};
    }
}

QHash<int, QByteArray> DitResultModel::roleNames() const
{
    auto roles = QAbstractTableModel::roleNames();
    roles[StatusRole] = "status";
    roles[StatusTextRole] = "statusText";
    roles[RelativePathRole] = "relativePath";
    roles[SizeARole] = "sizeA";
    roles[SizeBRole] = "sizeB";
    roles[ChecksumARole] = "checksumA";
    roles[ChecksumBRole] = "checksumB";
    roles[IsFolderRole] = "isFolder";
    return roles;
}

void DitResultModel::clear()
{
    setRows({});
}

void DitResultModel::setRows(QVector<DitResultRow> rows)
{
    beginResetModel();
    m_rows = std::move(rows);
    endResetModel();
}

const DitResultRow *DitResultModel::rowAt(int row) const
{
    if (row < 0 || row >= m_rows.size()) {
        return nullptr;
    }
    return &m_rows.at(row);
}

QString DitResultModel::statusText(int status)
{
    switch (status) {
    case SEDER_ROW_MATCHING: return QStringLiteral("MATCHING");
    case SEDER_ROW_CHANGED: return QStringLiteral("CHANGED");
    case SEDER_ROW_ONLY_IN_A: return QStringLiteral("ONLY A");
    case SEDER_ROW_ONLY_IN_B: return QStringLiteral("ONLY B");
    case SEDER_ROW_FOLDER_ONLY_IN_A: return QStringLiteral("FOLDER A");
    case SEDER_ROW_FOLDER_ONLY_IN_B: return QStringLiteral("FOLDER B");
    default: return QStringLiteral("UNKNOWN");
    }
}

QString DitResultModel::formatBytes(quint64 value)
{
    const char *units[] = {"B", "KB", "MB", "GB", "TB"};
    double scaled = static_cast<double>(value);
    int unit = 0;
    while (scaled >= 1024.0 && unit < 4) {
        scaled /= 1024.0;
        ++unit;
    }
    if (unit == 0) {
        return QStringLiteral("%1 B").arg(value);
    }
    return QStringLiteral("%1 %2").arg(scaled, 0, 'f', 1).arg(units[unit]);
}
