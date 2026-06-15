#include <QtTest>
#include "ClipLibraryModel.h"
#include "DestinationItem.h"
#include "DestinationListModel.h"
#include "JobQueueModel.h"

class DitModelTests : public QObject {
    Q_OBJECT

private slots:
    void destinationItemStateChanges();
    void destinationListModelAddRemove();
    void destinationListModelRoles();
    void destinationListModelEmitsDataChanged();
    void clipLibraryParsesMetadataJson();
    void clipLibraryFiltersAndClears();
    void jobQueueEnqueueRunAdvance();
    void jobQueueReorderAndClear();
};

static OffloadRequestData makeRequest(const QString &source, int destCount)
{
    OffloadRequestData r;
    r.sourcePath = source;
    for (int i = 0; i < destCount; ++i) {
        DestinationRequest dr;
        dr.path = QStringLiteral("%1/dest%2").arg(source).arg(i);
        r.destinations.append(dr);
    }
    return r;
}

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

void DitModelTests::jobQueueEnqueueRunAdvance()
{
    JobQueueModel queue;
    QCOMPARE(queue.count(), 0);
    QCOMPARE(queue.nextQueuedIndex(), -1);

    const int a = queue.enqueue(makeRequest(QStringLiteral("/cards/A"), 2), QStringLiteral("A"));
    const int b = queue.enqueue(makeRequest(QStringLiteral("/cards/B"), 1), QStringLiteral("B"));
    QCOMPARE(a, 0);
    QCOMPARE(b, 1);
    QCOMPARE(queue.count(), 2);
    QCOMPARE(queue.activeCount(), 2);

    QModelIndex i0 = queue.index(0);
    QCOMPARE(queue.data(i0, JobQueueModel::DestinationCountRole).toInt(), 2);
    QCOMPARE(queue.requestAt(0).destinations.size(), 2);

    // Run the first job, then advance to the second.
    QCOMPARE(queue.nextQueuedIndex(), 0);
    queue.setState(0, JobQueueModel::Running);
    QCOMPARE(queue.nextQueuedIndex(), 1);
    queue.setState(0, JobQueueModel::Complete);
    queue.setSummary(0, QStringLiteral("PASS"), 42);
    QCOMPARE(queue.data(i0, JobQueueModel::FinalStatusRole).toString(), QStringLiteral("PASS"));
    QCOMPARE(queue.data(i0, JobQueueModel::TotalFilesRole).toInt(), 42);
    QCOMPARE(queue.activeCount(), 1);
}

void DitModelTests::jobQueueReorderAndClear()
{
    JobQueueModel queue;
    queue.enqueue(makeRequest(QStringLiteral("/A"), 1), QStringLiteral("A"));
    queue.enqueue(makeRequest(QStringLiteral("/B"), 1), QStringLiteral("B"));
    queue.enqueue(makeRequest(QStringLiteral("/C"), 1), QStringLiteral("C"));

    queue.moveJob(0, 2); // A,B,C -> B,C,A
    QCOMPARE(queue.data(queue.index(0), JobQueueModel::LabelRole).toString(), QStringLiteral("B"));
    QCOMPARE(queue.data(queue.index(2), JobQueueModel::LabelRole).toString(), QStringLiteral("A"));

    queue.setState(0, JobQueueModel::Complete);
    queue.setState(1, JobQueueModel::Failed);
    queue.clearFinished();
    QCOMPARE(queue.count(), 1);
    QCOMPARE(queue.data(queue.index(0), JobQueueModel::LabelRole).toString(), QStringLiteral("A"));
}

QTEST_MAIN(DitModelTests)
#include "dit_model_tests.moc"
