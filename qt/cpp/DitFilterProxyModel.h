#pragma once

#include <QSortFilterProxyModel>

class DitFilterProxyModel final : public QSortFilterProxyModel {
    Q_OBJECT
    Q_PROPERTY(int filter READ filter WRITE setFilter NOTIFY filterChanged)
    Q_PROPERTY(int visibleRowCount READ visibleRowCount NOTIFY rowCountChanged)

public:
    enum Filter {
        All = 0,
        Matching = 1,
        Changed = 2,
        OnlyA = 3,
        OnlyB = 4,
        Folders = 5
    };
    Q_ENUM(Filter)

    explicit DitFilterProxyModel(QObject *parent = nullptr);

    int filter() const;
    void setFilter(int filter);
    int visibleRowCount() const;

signals:
    void filterChanged();
    void rowCountChanged();

protected:
    bool filterAcceptsRow(int sourceRow, const QModelIndex &sourceParent) const override;

private:
    int m_filter = All;
};
