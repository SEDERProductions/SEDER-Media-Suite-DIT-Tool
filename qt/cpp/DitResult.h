#pragma once

#include <QString>
#include <QVector>
#include <QMetaType>
#include <QtGlobal>

struct DitRequestData {
    QString sourcePath;
    QString destinationPath;
    QString projectName;
    QString shootDate;
    QString cardName;
    QString cameraId;
    QString ignorePatterns;
    int compareMode = 2;
    bool ignoreHiddenSystem = true;
};

struct DitProgress {
    QString phase;
    quint64 processedFiles = 0;
    quint64 processedBytes = 0;
    QString status;
};

struct DitResultRow {
    int status = 0;
    QString relativePath;
    bool hasSizeA = false;
    bool hasSizeB = false;
    quint64 sizeA = 0;
    quint64 sizeB = 0;
    QString checksumA;
    QString checksumB;
    bool folder = false;
};

struct DitSummaryData {
    quint64 onlyA = 0;
    quint64 onlyB = 0;
    quint64 changed = 0;
    quint64 matching = 0;
    quint64 totalFiles = 0;
    quint64 totalFolders = 0;
    quint64 totalSize = 0;
    bool pass = false;
    bool mhlAvailable = false;
    int compareMode = 2;
};

struct DitResult {
    QVector<DitResultRow> rows;
    DitSummaryData summary;
    QString txtExport;
    QString csvExport;
    QString mhlExport;
};

Q_DECLARE_METATYPE(DitProgress)
Q_DECLARE_METATYPE(DitResult)
