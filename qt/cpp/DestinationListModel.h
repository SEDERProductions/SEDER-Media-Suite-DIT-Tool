#pragma once

#include "DestinationItem.h"
#include <QAbstractListModel>
#include <QVector>

class DestinationListModel : public QAbstractListModel {
    Q_OBJECT
    Q_PROPERTY(int count READ count NOTIFY countChanged)

public:
    enum Roles {
        PathRole = Qt::UserRole + 1,
        LabelRole,
        StateRole,
        ProgressRole,
        ErrorRole
    };
    Q_ENUM(Roles)

    explicit DestinationListModel(QObject *parent = nullptr);

    int rowCount(const QModelIndex &parent = QModelIndex()) const override;
    QVariant data(const QModelIndex &index, int role = Qt::DisplayRole) const override;
    QHash<int, QByteArray> roleNames() const override;

    int count() const;

    Q_INVOKABLE void addDestination(const QString &path, const QString &label = QString());
    Q_INVOKABLE void removeDestination(int index);
    Q_INVOKABLE void clear();
    Q_INVOKABLE QStringList paths() const;

    QVector<DestinationItem *> items() const { return m_items; }

signals:
    void countChanged();

private:
    QVector<DestinationItem *> m_items;
};
