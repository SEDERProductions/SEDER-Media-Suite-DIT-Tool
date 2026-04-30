#include "DitFilterProxyModel.h"
#include "DitResultModel.h"
#include "seder_ffi.h"

#include <QtTest/QtTest>
#include <utility>

class DitModelTests final : public QObject {
    Q_OBJECT

private slots:
    void loadsLargeResultSets();
    void filtersStatusesAndFolders();
};

void DitModelTests::loadsLargeResultSets()
{
    for (const int count : {1000, 10000, 100000}) {
        QVector<DitResultRow> rows;
        rows.reserve(count);
        for (int i = 0; i < count; ++i) {
            DitResultRow row;
            row.status = i % 5 == 0 ? SEDER_ROW_CHANGED : SEDER_ROW_MATCHING;
            row.relativePath = QStringLiteral("A001/clip-%1.mov").arg(i, 6, 10, QLatin1Char('0'));
            row.hasSizeA = true;
            row.hasSizeB = true;
            row.sizeA = 1024;
            row.sizeB = row.status == SEDER_ROW_MATCHING ? 1024 : 2048;
            rows.push_back(std::move(row));
        }

        DitResultModel model;
        model.setRows(std::move(rows));
        QCOMPARE(model.rowCount(), count);
        QCOMPARE(model.columnCount(), 6);
        QVERIFY(model.index(count - 1, 1).data().toString().startsWith(QStringLiteral("A001/clip-")));
    }
}

void DitModelTests::filtersStatusesAndFolders()
{
    QVector<DitResultRow> rows;
    DitResultRow matching;
    matching.status = SEDER_ROW_MATCHING;
    matching.relativePath = QStringLiteral("same.mov");
    rows.push_back(matching);

    DitResultRow changed;
    changed.status = SEDER_ROW_CHANGED;
    changed.relativePath = QStringLiteral("changed.mov");
    rows.push_back(changed);

    DitResultRow onlyA;
    onlyA.status = SEDER_ROW_ONLY_IN_A;
    onlyA.relativePath = QStringLiteral("only-a.mov");
    rows.push_back(onlyA);

    DitResultRow folder;
    folder.status = SEDER_ROW_FOLDER_ONLY_IN_B;
    folder.relativePath = QStringLiteral("missing-folder");
    folder.folder = true;
    rows.push_back(folder);

    DitResultModel model;
    model.setRows(rows);
    DitFilterProxyModel proxy;
    proxy.setSourceModel(&model);

    proxy.setFilter(DitFilterProxyModel::Matching);
    QCOMPARE(proxy.rowCount(), 1);

    proxy.setFilter(DitFilterProxyModel::Changed);
    QCOMPARE(proxy.rowCount(), 1);

    proxy.setFilter(DitFilterProxyModel::OnlyA);
    QCOMPARE(proxy.rowCount(), 1);

    proxy.setFilter(DitFilterProxyModel::OnlyB);
    QCOMPARE(proxy.rowCount(), 1);

    proxy.setFilter(DitFilterProxyModel::Folders);
    QCOMPARE(proxy.rowCount(), 1);
}

QTEST_MAIN(DitModelTests)
#include "dit_model_tests.moc"
