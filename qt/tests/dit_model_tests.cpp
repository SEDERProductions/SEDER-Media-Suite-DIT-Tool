#include <QtTest>
#include "ClipLibraryModel.h"
#include "DestinationItem.h"
#include "DestinationListModel.h"

class DitModelTests : public QObject {
    Q_OBJECT

private slots:
    void destinationItemStateChanges();
    void destinationListModelAddRemove();
    void destinationListModelRoles();
    void destinationListModelEmitsDataChanged();
    void clipLibraryParsesMetadataJson();
    void clipLibraryFiltersAndClears();
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

namespace {
const char *kSampleMetadataJson = R"({
  "files": [
    {
      "path": "A001/clip001.mxf",
      "size": 1048576,
      "hash_algorithm": "BLAKE3",
      "hash": "abc123",
      "media_kind": "MXF",
      "metadata": {
        "video_codec": "prores", "width": 1920, "height": 1080,
        "fps_num": 24000, "fps_den": 1001, "duration_seconds": 10.5,
        "audio_codec": "pcm_s16le", "audio_channels": 2, "audio_sample_rate": 48000,
        "timecode": "01:00:00:00", "color_space": "bt709"
      }
    },
    {
      "path": "A001/audio.wav",
      "size": 2048,
      "hash_algorithm": "BLAKE3",
      "hash": "def456",
      "media_kind": "Audio",
      "metadata": null
    }
  ],
  "ignored_paths": []
})";
}

void DitModelTests::clipLibraryParsesMetadataJson()
{
    ClipLibraryModel model;
    QCOMPARE(model.count(), 0);

    model.loadFromMetadataJson(QByteArray(kSampleMetadataJson), QStringLiteral("/Volumes/CARD"));
    QCOMPARE(model.count(), 2);
    QCOMPARE(model.sourcePath(), QStringLiteral("/Volumes/CARD"));

    const QModelIndex first = model.index(0);
    QCOMPARE(model.data(first, ClipLibraryModel::FileNameRole).toString(), QStringLiteral("clip001.mxf"));
    QCOMPARE(model.data(first, ClipLibraryModel::CodecRole).toString(), QStringLiteral("prores"));
    QCOMPARE(model.data(first, ClipLibraryModel::ResolutionRole).toString(), QStringLiteral("1920 × 1080"));
    QCOMPARE(model.data(first, ClipLibraryModel::FpsRole).toString(), QStringLiteral("23.976 fps"));
    QCOMPARE(model.data(first, ClipLibraryModel::DurationRole).toString(), QStringLiteral("00:00:11"));
    QCOMPARE(model.data(first, ClipLibraryModel::TimecodeRole).toString(), QStringLiteral("01:00:00:00"));
    QCOMPARE(model.data(first, ClipLibraryModel::HasMetadataRole).toBool(), true);

    const QModelIndex second = model.index(1);
    QCOMPARE(model.data(second, ClipLibraryModel::FileNameRole).toString(), QStringLiteral("audio.wav"));
    QCOMPARE(model.data(second, ClipLibraryModel::HasMetadataRole).toBool(), false);
    QCOMPARE(model.data(second, ClipLibraryModel::SizeTextRole).toString(), QStringLiteral("2.00 KB"));
}

void DitModelTests::clipLibraryFiltersAndClears()
{
    ClipLibraryModel model;
    model.loadFromMetadataJson(QByteArray(kSampleMetadataJson), QStringLiteral("/src"));

    QCOMPARE(model.items().size(), 2);
    QCOMPARE(model.items(QStringLiteral("clip001")).size(), 1);
    QCOMPARE(model.items(QStringLiteral("prores")).size(), 1);
    QCOMPARE(model.items(QStringLiteral("nomatch")).size(), 0);

    const QVariantMap row = model.get(0);
    QCOMPARE(row.value(QStringLiteral("fileName")).toString(), QStringLiteral("clip001.mxf"));

    model.loadFromMetadataJson(QByteArray(), QString());
    QCOMPARE(model.count(), 0);
}

QTEST_MAIN(DitModelTests)
#include "dit_model_tests.moc"
