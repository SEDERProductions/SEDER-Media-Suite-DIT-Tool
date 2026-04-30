#pragma once

#include "DitResult.h"

#include <QObject>

class DitCompareWorker final : public QObject {
    Q_OBJECT

public:
    explicit DitCompareWorker(DitRequestData request, QObject *parent = nullptr);

public slots:
    void run();

signals:
    void progress(const DitProgress &progress);
    void finished(const DitResult &result);
    void failed(const QString &message);

private:
    DitRequestData m_request;
};
