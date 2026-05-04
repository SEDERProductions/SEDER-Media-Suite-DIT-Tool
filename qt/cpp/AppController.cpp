#include "AppController.h"

#include <QFileDialog>
#include <QSaveFile>
#include <QThread>
#include <functional>

namespace {

template<typename T>
void setIfChanged(T &field, const T &value, const std::function<void()> &notify)
{
    if (field == value) return;
    field = value;
    notify();
}

} // namespace

AppController::AppController(QObject *parent)
    : QObject(parent)
    , m_destinationModel(new DestinationListModel(this))
{
    appendLog(QStringLiteral("Ready to offload media."));
}

QString AppController::sourcePath() const { return m_sourcePath; }
void AppController::setSourcePath(const QString &value) { setIfChanged(m_sourcePath, value, [this] { emit sourcePathChanged(); }); }
DestinationListModel *AppController::destinationModel() const { return m_destinationModel; }
QString AppController::projectName() const { return m_projectName; }
void AppController::setProjectName(const QString &value) { setIfChanged(m_projectName, value, [this] { emit projectNameChanged(); }); }
QString AppController::shootDate() const { return m_shootDate; }
void AppController::setShootDate(const QString &value) { setIfChanged(m_shootDate, value, [this] { emit shootDateChanged(); }); }
QString AppController::cardName() const { return m_cardName; }
void AppController::setCardName(const QString &value) { setIfChanged(m_cardName, value, [this] { emit cardNameChanged(); }); }
QString AppController::cameraId() const { return m_cameraId; }
void AppController::setCameraId(const QString &value) { setIfChanged(m_cameraId, value, [this] { emit cameraIdChanged(); }); }
QString AppController::ignorePatterns() const { return m_ignorePatterns; }
void AppController::setIgnorePatterns(const QString &value) { setIfChanged(m_ignorePatterns, value, [this] { emit ignorePatternsChanged(); }); }
bool AppController::ignoreHiddenSystem() const { return m_ignoreHiddenSystem; }
void AppController::setIgnoreHiddenSystem(bool value) { if (m_ignoreHiddenSystem != value) { m_ignoreHiddenSystem = value; emit ignoreHiddenSystemChanged(); } }
bool AppController::verifyAfterCopy() const { return m_verifyAfterCopy; }
void AppController::setVerifyAfterCopy(bool value) { if (m_verifyAfterCopy != value) { m_verifyAfterCopy = value; emit verifyAfterCopyChanged(); } }
bool AppController::busy() const { return m_busy; }
double AppController::overallProgress() const { return m_overallProgress; }
QString AppController::statusText() const { return m_statusText; }
QString AppController::currentFile() const { return m_currentFile; }
QStringList AppController::logLines() const { return m_logLines; }
bool AppController::canExport() const { return m_canExport; }
quint64 AppController::totalFiles() const { return m_totalFiles; }
quint64 AppController::totalSize() const { return m_totalSize; }
bool AppController::pass() const { return m_pass; }

void AppController::chooseSourceFolder()
{
    const QString path = QFileDialog::getExistingDirectory(nullptr, tr("Choose Source Folder"), m_sourcePath);
    if (!path.isEmpty()) {
        setSourcePath(path);
    }
}

void AppController::addDestinationFolder()
{
    const QString path = QFileDialog::getExistingDirectory(nullptr, tr("Choose Destination Folder"), QString());
    if (!path.isEmpty()) {
        m_destinationModel->addDestination(path);
    }
}

void AppController::removeDestination(int index)
{
    m_destinationModel->removeDestination(index);
}

void AppController::startOffload()
{
    if (m_busy) return;
    if (m_sourcePath.isEmpty()) {
        setStatusText(QStringLiteral("Missing source folder."));
        appendLog(QStringLiteral("Source folder is required."));
        return;
    }
    if (m_destinationModel->count() == 0) {
        setStatusText(QStringLiteral("No destinations selected."));
        appendLog(QStringLiteral("At least one destination is required."));
        return;
    }

    OffloadRequestData request;
    request.sourcePath = m_sourcePath;
    for (auto *item : m_destinationModel->items()) {
        DestinationRequest dr;
        dr.path = item->path();
        dr.label = item->label();
        request.destinations.append(dr);
    }
    request.projectName = m_projectName;
    request.shootDate = m_shootDate;
    request.cardName = m_cardName;
    request.cameraId = m_cameraId;
    request.ignorePatterns = m_ignorePatterns;
    request.ignoreHiddenSystem = m_ignoreHiddenSystem;
    request.verifyAfterCopy = m_verifyAfterCopy;

    setBusy(true);
    setOverallProgress(0.0);
    setStatusText(QStringLiteral("Starting offload..."));
    setCurrentFile(QString());
    setPass(false);
    m_canExport = false;
    emit exportStateChanged();
    emit summaryChanged();
    appendLog(QStringLiteral("Starting DIT offload."));

    for (auto *item : m_destinationModel->items()) {
        item->setState(DestinationItem::Pending);
        item->setProgress(0.0);
        item->setError(QString());
    }

    auto *thread = new QThread(this);
    auto *worker = new DitOffloadWorker(request);
    m_activeWorker = worker;
    worker->moveToThread(thread);

    connect(thread, &QThread::started, worker, &DitOffloadWorker::run);
    connect(worker, &DitOffloadWorker::progress, this, [this](const OffloadProgressData &update) {
        setStatusText(update.phase);
        setCurrentFile(update.currentFile);
        if (update.overallBytesTotal > 0) {
            setOverallProgress(static_cast<double>(update.overallBytesCompleted) / static_cast<double>(update.overallBytesTotal));
        }
        for (int i = 0; i < update.destinations.size() && i < m_destinationModel->count(); ++i) {
            const DestinationProgressData &dpd = update.destinations.at(i);
            auto *item = m_destinationModel->items().at(i);
            item->setState(static_cast<int>(dpd.state));
            if (dpd.bytesTotal > 0) {
                item->setProgress(static_cast<double>(dpd.bytesCompleted) / static_cast<double>(dpd.bytesTotal));
            }
            if (!dpd.error.isEmpty()) {
                item->setError(dpd.error);
            }
        }
    });
    connect(worker, &DitOffloadWorker::finished, this, [this] {
        setBusy(false);
        setOverallProgress(1.0);
        setStatusText(QStringLiteral("Offload complete."));
        setPass(true);
        m_canExport = true;
        emit exportStateChanged();
        emit summaryChanged();
        appendLog(QStringLiteral("Offload complete."));
    });
    connect(worker, &DitOffloadWorker::failed, this, [this](const QString &message) {
        setBusy(false);
        setOverallProgress(0.0);
        setStatusText(message);
        setPass(false);
        appendLog(QStringLiteral("Offload failed: %1").arg(message));
    });
    connect(worker, &DitOffloadWorker::cancelled, this, [this] {
        setBusy(false);
        setOverallProgress(0.0);
        setStatusText(QStringLiteral("Offload cancelled."));
        setPass(false);
        appendLog(QStringLiteral("Offload cancelled by user."));
    });
    connect(worker, &DitOffloadWorker::finished, thread, &QThread::quit);
    connect(worker, &DitOffloadWorker::failed, thread, &QThread::quit);
    connect(worker, &DitOffloadWorker::cancelled, thread, &QThread::quit);
    connect(thread, &QThread::finished, worker, &QObject::deleteLater);
    connect(thread, &QThread::finished, thread, &QObject::deleteLater);
    connect(thread, &QThread::finished, this, [this] { m_activeWorker = nullptr; });
    thread->start();
}

void AppController::cancelOffload()
{
    if (m_activeWorker) {
        m_activeWorker->cancel();
        appendLog(QStringLiteral("Cancelling offload..."));
    }
}

void AppController::exportTxt()
{
    // TODO: Wire up to actual report handle
    writeExport(tr("Export TXT Report"), QStringLiteral("seder-dit-report.txt"), m_txtExport);
}

void AppController::exportCsv()
{
    writeExport(tr("Export CSV Report"), QStringLiteral("seder-dit-report.csv"), m_csvExport);
}

void AppController::exportMhl()
{
    writeExport(tr("Export MHL Report"), QStringLiteral("seder-dit-report.mhl"), m_mhlExport);
}

QString AppController::formatBytes(quint64 value) const
{
    const QStringList units = { QStringLiteral("B"), QStringLiteral("KB"), QStringLiteral("MB"), QStringLiteral("GB"), QStringLiteral("TB") };
    if (value == 0) return QStringLiteral("0 B");
    int exp = static_cast<int>(std::min(static_cast<size_t>(std::log(static_cast<double>(value)) / std::log(1024.0)), static_cast<size_t>(units.size() - 1)));
    double scaled = static_cast<double>(value) / std::pow(1024.0, exp);
    if (exp == 0) return QStringLiteral("%1 %2").arg(value).arg(units[exp]);
    return QStringLiteral("%1 %2").arg(scaled, 0, 'f', 2).arg(units[exp]);
}

void AppController::appendLog(const QString &line)
{
    m_logLines.append(line);
    while (m_logLines.size() > 300) {
        m_logLines.removeFirst();
    }
    emit logLinesChanged();
}

void AppController::setBusy(bool value)
{
    if (m_busy == value) return;
    m_busy = value;
    emit busyChanged();
}

void AppController::setOverallProgress(double value)
{
    value = qBound(0.0, value, 1.0);
    if (qFuzzyCompare(m_overallProgress, value)) return;
    m_overallProgress = value;
    emit overallProgressChanged();
}

void AppController::setStatusText(const QString &value)
{
    if (m_statusText == value) return;
    m_statusText = value;
    emit statusTextChanged();
}

void AppController::setCurrentFile(const QString &value)
{
    if (m_currentFile == value) return;
    m_currentFile = value;
    emit currentFileChanged();
}

void AppController::setPass(bool value)
{
    if (m_pass == value) return;
    m_pass = value;
    emit summaryChanged();
}

void AppController::writeExport(const QString &caption, const QString &defaultName, const QString &contents)
{
    if (contents.isEmpty()) {
        setStatusText(QStringLiteral("No report is ready to export."));
        appendLog(QStringLiteral("Export skipped because no report is ready."));
        return;
    }
    const QString path = QFileDialog::getSaveFileName(nullptr, caption, defaultName);
    if (path.isEmpty()) {
        appendLog(QStringLiteral("Export canceled."));
        return;
    }
    QSaveFile file(path);
    if (!file.open(QIODevice::WriteOnly | QIODevice::Text)) {
        setStatusText(QStringLiteral("Unable to write %1").arg(path));
        appendLog(QStringLiteral("Unable to write %1").arg(path));
        return;
    }
    file.write(contents.toUtf8());
    if (!file.commit()) {
        setStatusText(QStringLiteral("Unable to save %1").arg(path));
        appendLog(QStringLiteral("Unable to save %1").arg(path));
        return;
    }
    setStatusText(QStringLiteral("Export complete."));
    appendLog(QStringLiteral("Exported %1").arg(path));
}
