#include <QtTest>
#include "DestinationItem.h"
#include "DestinationListModel.h"
#define private public
#include "DitOffloadWorker.h"
#undef private
#include "seder_ffi.h"

class DitModelTests : public QObject {
    Q_OBJECT

private slots:
    void destinationItemStateChanges();
    void destinationListModelAddRemove();
    void destinationListModelRoles();
    void destinationListModelEmitsDataChanged();
    void offloadProgressNullDestinationErrorDoesNotCrashAndIsEmpty();
};

void DitModelTests::destinationItemStateChanges()
{
    DestinationItem item;
    QCOMPARE(item.state(), static_cast<int>(DestinationItem::Pending));

    item.setState(DestinationItem::Copying);
    QCOMPARE(item.state(), static_cast<int>(DestinationItem::Copying));

    item.setProgress(0.5);
    QCOMPARE(item.progress(), 0.5);

    item.setError(QStringLiteral("Test error"));
    QCOMPARE(item.error(), QStringLiteral("Test error"));
}

void DitModelTests::destinationListModelAddRemove()
{
    DestinationListModel model;
    QCOMPARE(model.count(), 0);

    model.addDestination(QStringLiteral("/path/one"), QStringLiteral("Drive A"));
    QCOMPARE(model.count(), 1);

    model.addDestination(QStringLiteral("/path/two"));
    QCOMPARE(model.count(), 2);

    model.removeDestination(0);
    QCOMPARE(model.count(), 1);

    model.clear();
    QCOMPARE(model.count(), 0);
}

void DitModelTests::destinationListModelRoles()
{
    DestinationListModel model;
    model.addDestination(QStringLiteral("/test/path"), QStringLiteral("Test"));

    QModelIndex idx = model.index(0);
    QCOMPARE(model.data(idx, DestinationListModel::PathRole).toString(), QStringLiteral("/test/path"));
    QCOMPARE(model.data(idx, DestinationListModel::LabelRole).toString(), QStringLiteral("Test"));
    QCOMPARE(model.data(idx, DestinationListModel::StateRole).toInt(), static_cast<int>(DestinationItem::Pending));
}

void DitModelTests::destinationListModelEmitsDataChanged()
{
    DestinationListModel model;
    model.addDestination(QStringLiteral("/test/path"), QStringLiteral("Test"));
    QSignalSpy spy(&model, &DestinationListModel::dataChanged);

    model.items().at(0)->setState(DestinationItem::Copying);

    QCOMPARE(spy.count(), 1);
    const auto args = spy.takeFirst();
    QCOMPARE(args.at(0).toModelIndex().row(), 0);
    QCOMPARE(args.at(1).toModelIndex().row(), 0);
    const QList<int> roles = args.at(2).value<QList<int>>();
    QCOMPARE(roles, QList<int>{DestinationListModel::StateRole});
}

void DitModelTests::offloadProgressNullDestinationErrorDoesNotCrashAndIsEmpty()
{
    DitOffloadWorker worker(OffloadRequestData{});
    bool signalReceived = false;
    OffloadProgressData captured;
    QObject::connect(&worker, &DitOffloadWorker::progress,
                     [&signalReceived, &captured](const OffloadProgressData &progressData) {
                         signalReceived = true;
                         captured = progressData;
                     });

    const SederDestinationProgress dests[] = {
        {
            1,
            2,
            3,
            4,
            5,
            "clip001.mov",
            0,
            nullptr
        }
    };

    const SederOffloadProgress progress{
        "copying",
        10,
        20,
        30,
        40,
        "clip001.mov",
        nullptr,
        dests,
        1
    };

    DitOffloadWorker::progressTrampoline(&progress, &worker);

    QCOMPARE(signalReceived, true);
    QCOMPARE(captured.destinations.size(), 1);
    QCOMPARE(captured.destinations.at(0).error, QString());
}

QTEST_MAIN(DitModelTests)
#include "dit_model_tests.moc"
