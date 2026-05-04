#pragma once

#include <QObject>
#include <QAtomicInteger>
#include <QStringList>

struct DestinationRequest {
    QString path;
    QString label;
};

struct OffloadRequestData {
    QString sourcePath;
    QVector<DestinationRequest> destinations;
    QString projectName;
    QString shootDate;
    QString cardName;
    QString cameraId;
    QString ignorePatterns;
    bool ignoreHiddenSystem = true;
    bool verifyAfterCopy = true;
};

struct DestinationProgressData {
    uint32_t state = 0;
    uint64_t filesCompleted = 0;
    uint64_t filesTotal = 0;
    uint64_t bytesCompleted = 0;
    uint64_t bytesTotal = 0;
    QString currentFile;
    QString error;
};

struct OffloadProgressData {
    QString phase;
    uint64_t overallFilesCompleted = 0;
    uint64_t overallFilesTotal = 0;
    uint64_t overallBytesCompleted = 0;
    uint64_t overallBytesTotal = 0;
    QString currentFile;
    QVector<DestinationProgressData> destinations;
};

class DitOffloadWorker final : public QObject {
    Q_OBJECT

public:
    explicit DitOffloadWorker(OffloadRequestData request, QObject *parent = nullptr);

    void cancel();

public slots:
    void run();

signals:
    void progress(const OffloadProgressData &progress);
    void finished();
    void failed(const QString &message);
    void cancelled();

private:
    OffloadRequestData m_request;
    QAtomicInteger<quint8> m_cancelToken{0};
};
