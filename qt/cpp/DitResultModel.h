#pragma once

#include "DitResult.h"

#include <QAbstractTableModel>

class DitResultModel final : public QAbstractTableModel {
    Q_OBJECT

public:
    enum Roles {
        StatusRole = Qt::UserRole + 1,
        StatusTextRole,
        RelativePathRole,
        SizeARole,
        SizeBRole,
        ChecksumARole,
        ChecksumBRole,
        IsFolderRole
    };
    Q_ENUM(Roles)

    explicit DitResultModel(QObject *parent = nullptr);

    int rowCount(const QModelIndex &parent = QModelIndex()) const override;
    int columnCount(const QModelIndex &parent = QModelIndex()) const override;
    QVariant data(const QModelIndex &index, int role = Qt::DisplayRole) const override;
    QVariant headerData(int section, Qt::Orientation orientation, int role) const override;
    QHash<int, QByteArray> roleNames() const override;

    Q_INVOKABLE void clear();
    void setRows(QVector<DitResultRow> rows);
    const DitResultRow *rowAt(int row) const;

    static QString statusText(int status);
    static QString formatBytes(quint64 value);

private:
    QVector<DitResultRow> m_rows;
};
