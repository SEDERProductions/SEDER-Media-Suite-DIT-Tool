#include <QtTest>
#include <QUrl>
#include "DestinationItem.h"
#include "DestinationListModel.h"
#include "MediaListModel.h"

class DitModelTests : public QObject {
    Q_OBJECT

private slots:
    void destinationItemStateChanges();
    void destinationListModelAddRemove();
    void destinationListModelRoles();
    void destinationListModelEmitsDataChanged();
    void mediaListModelParsesJsonAndClassifies();
    void mediaListModelSetThumbnailEmitsDataChanged();
    void mediaListModelStaleThumbnailIsIgnored();
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

void DitModelTests::mediaListModelParsesJsonAndClassifies()
{
    MediaListModel model;
    QCOMPARE(model.count(), 0);

    const QByteArray json = R"([
        {"rel_path":"A001.mxf","abs_path":"/src/A001.mxf","size":1000,"kind":"MXF"},
        {"rel_path":"sound/audio.wav","abs_path":"/src/sound/audio.wav","size":50,"kind":"Audio"}
    ])";
    model.resetFromJson(json);
    QCOMPARE(model.count(), 2);

    const QModelIndex video = model.index(0);
    QCOMPARE(model.data(video, MediaListModel::FileNameRole).toString(), QStringLiteral("A001.mxf"));
    QCOMPARE(model.data(video, MediaListModel::KindRole).toString(), QStringLiteral("MXF"));
    QCOMPARE(model.data(video, MediaListModel::ThumbnailStateRole).toInt(),
             static_cast<int>(MediaListModel::NoThumb));

    const QModelIndex audio = model.index(1);
    // fileName is the last path component; audio is a non-visual kind.
    QCOMPARE(model.data(audio, MediaListModel::FileNameRole).toString(), QStringLiteral("audio.wav"));
    QCOMPARE(model.data(audio, MediaListModel::ThumbnailStateRole).toInt(),
             static_cast<int>(MediaListModel::Unsupported));

    model.clear();
    QCOMPARE(model.count(), 0);
}

void DitModelTests::mediaListModelSetThumbnailEmitsDataChanged()
{
    MediaListModel model;
    model.resetFromJson(R"([{"rel_path":"A001.mxf","abs_path":"/src/A001.mxf","size":10,"kind":"MXF"}])");
    QSignalSpy spy(&model, &MediaListModel::dataChanged);

    model.setThumbnail(QStringLiteral("/src/A001.mxf"),
                       QUrl::fromLocalFile(QStringLiteral("/cache/x.jpg")),
                       MediaListModel::Ready);

    QCOMPARE(spy.count(), 1);
    const QModelIndex idx = model.index(0);
    QCOMPARE(model.data(idx, MediaListModel::ThumbnailStateRole).toInt(),
             static_cast<int>(MediaListModel::Ready));
    QVERIFY(model.data(idx, MediaListModel::ThumbnailUrlRole).toUrl().isValid());
}

void DitModelTests::mediaListModelStaleThumbnailIsIgnored()
{
    MediaListModel model;
    model.resetFromJson(R"([{"rel_path":"A001.mxf","abs_path":"/src/A001.mxf","size":10,"kind":"MXF"}])");
    QSignalSpy spy(&model, &MediaListModel::dataChanged);

    // A result for a path from a previous source must be a harmless no-op.
    model.setThumbnail(QStringLiteral("/old/gone.mxf"),
                       QUrl::fromLocalFile(QStringLiteral("/cache/y.jpg")),
                       MediaListModel::Ready);

    QCOMPARE(spy.count(), 0);
}

QTEST_MAIN(DitModelTests)
#include "dit_model_tests.moc"
