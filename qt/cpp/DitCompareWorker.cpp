#include "DitCompareWorker.h"

#include "SederFfi.h"

#include <exception>
#include <utility>

DitCompareWorker::DitCompareWorker(DitRequestData request, QObject *parent)
    : QObject(parent)
    , m_request(std::move(request))
{
}

void DitCompareWorker::run()
{
    try {
        DitResult result = SederFfi::compare(m_request, [this](const DitProgress &update) {
            emit progress(update);
        });
        emit finished(result);
    } catch (const std::exception &err) {
        emit failed(QString::fromUtf8(err.what()));
    } catch (...) {
        emit failed(QStringLiteral("DIT comparison failed."));
    }
}
