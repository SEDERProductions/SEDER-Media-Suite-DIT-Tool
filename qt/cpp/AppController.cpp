#include "AppController.h"

#include <QFileDialog>
#include <QFileInfo>
#include <QGuiApplication>
#include <QClipboard>
#include <QDateTime>
#include <QSaveFile>
#include <QThread>
#include <QRegularExpression>
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
void AppController::setProjectName(const QString &value) { setIfChanged(m_projectName, value.trimmed(), [this] { emit projectNameChanged(); }); }
QString AppController::shootDate() const { return m_shootDate; }
void AppController::setShootDate(const QString &value) { setIfChanged(m_shootDate, value.trimmed(), [this] { emit shootDateChanged(); }); }
QString AppController::cardName() const { return m_cardName; }
void AppController::setCardName(const QString &value) { setIfChanged(m_cardName, value.trimmed(), [this] { emit cardNameChanged(); }); }
QString AppController::cameraId() const { return m_cameraId; }
void AppController::setCameraId(const QString &value) { setIfChanged(m_cameraId, value.trimmed(), [this] { emit cameraIdChanged(); }); }
QString AppController::ignorePatterns() const { return m_ignorePatterns; }
void AppController::setIgnorePatterns(const QString &value) { setIfChanged(m_ignorePatterns, value, [this] { emit ignorePatternsChanged(); }); }
bool AppController::ignoreHiddenSystem() const { return m_ignoreHiddenSystem; }
void AppController::setIgnoreHiddenSystem(bool value) { if (m_ignoreHiddenSystem != value) { m_ignoreHiddenSystem = value; emit ignoreHiddenSystemChanged(); } }
bool AppController::verifyAfterCopy() const { return m_verifyAfterCopy; }
void AppController::setVerifyAfterCopy(bool value) { if (m_verifyAfterCopy != value) { m_verifyAfterCopy = value; emit verifyAfterCopyChanged(); } }
bool AppController::skipExisting() const { return m_skipExisting; }
void AppController::setSkipExisting(bool value) { if (m_skipExisting != value) { m_skipExisting = value; emit skipExistingChanged(); } }
bool AppController::generateReport() const { return m_generateReport; }
void AppController::setGenerateReport(bool value) { if (m_generateReport != value) { m_generateReport = value; emit generateReportChanged(); } }
bool AppController::busy() const { return m_busy; }
double AppController::overallProgress() const { return m_overallProgress; }
QString AppController::statusText() const { return m_statusText; }
QString AppController::currentFile() const { return m_currentFile; }
QStringList AppController::logLines() const { return m_logLines; }
bool AppController::canExport() const { return m_canExport; }
bool AppController::canExportMhl() const { return m_canExport && m_canExportMhl && !m_mhlExport.isEmpty(); }
QString AppController::finalStatus() const { return m_finalStatus; }
bool AppController::verificationPerformed() const { return m_verificationPerformed; }
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

void AppController::syncDestinationPaths()
{
    if (m_sourcePath.isEmpty() || m_destinationModel->count() == 0) return;
    const QString newTail = QFileInfo(m_sourcePath).fileName();
    if (newTail.isEmpty()) return;
    for (auto *item : m_destinationModel->items()) {
        const QFileInfo fi(item->path());
        const QString parent = fi.absolutePath();
        const QString newPath = parent + QDir::separator() + newTail;
        if (newPath != item->path()) {
            item->setPath(newPath);
        }
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
        appendLog(QStringLiteral("Source folder is required."), LogSeverity::Warn);
        return;
    }
    if (m_destinationModel->count() == 0) {
        setStatusText(QStringLiteral("No destinations selected."));
        appendLog(QStringLiteral("At least one destination is required."), LogSeverity::Warn);
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
    static const QRegularExpression shootDatePattern(QStringLiteral(R"(^\d{4}-\d{2}-\d{2}$)"));
    if (!request.shootDate.isEmpty() && !shootDatePattern.match(request.shootDate).hasMatch()) {
        appendLog(QStringLiteral("Warning: shoot date '%1' should use YYYY-MM-DD format.").arg(request.shootDate));
    }
    request.cardName = m_cardName;
    request.cameraId = m_cameraId;
    request.ignorePatterns = m_ignorePatterns;
    request.ignoreHiddenSystem = m_ignoreHiddenSystem;
    request.verifyAfterCopy = m_verifyAfterCopy;
    request.skipExisting = m_skipExisting;
    request.generateReport = m_generateReport;

    setBusy(true);
    setOverallProgress(0.0);
    setStatusText(QStringLiteral("Starting offload..."));
    setCurrentFile(QString());
    setPass(false);
    m_canExport = false;
    m_canExportMhl = false;
    m_mhlExport.clear();
    m_finalStatus = QStringLiteral("FAIL");
    m_verificationPerformed = false;
    emit exportStateChanged();
    emit canExportMhlChanged();
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
        const bool isScanningPhase = update.phase.startsWith(QStringLiteral("scanning_source"));
        if (update.phase == QStringLiteral("scanning_source_start")) {
            setStatusText(QStringLiteral("Scanning source..."));
        } else if (update.phase == QStringLiteral("scanning_source_complete")) {
            setStatusText(QStringLiteral("Source scan complete. Starting copy..."));
        } else if (update.phase == QStringLiteral("scanning_source")) {
            setStatusText(QStringLiteral("Scanning source..."));
        } else if (update.phase == QStringLiteral("verifying")) {
            setStatusText(QStringLiteral("Verifying copies..."));
        } else {
            setStatusText(QStringLiteral("Copying files..."));
        }
        setCurrentFile(update.currentFile);
        // Handle warnings from the engine
        if (!update.warning.isEmpty()) {
            appendLog(update.warning, LogSeverity::Warn);
        }
        if (isScanningPhase) {
            if (update.overallFilesTotal > 0) {
                setOverallProgress(static_cast<double>(update.overallFilesCompleted) / static_cast<double>(update.overallFilesTotal));
            } else {
                setOverallProgress(0.0);
            }
        } else if (update.overallBytesTotal > 0) {
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
    connect(worker, &DitOffloadWorker::finished, this, [this, request](const FinalReportData &report) {
        setBusy(false);
        setOverallProgress(1.0);
        setPass(report.allPass);
        if (report.allPass) {
            setStatusText(QStringLiteral("Offload complete."));
            appendLog(QStringLiteral("Offload complete."));
        } else {
            setStatusText(QStringLiteral("Offload completed with errors."));
            appendLog(QStringLiteral("Offload completed with errors."));
        }
        m_totalFiles = report.totalFiles;
        m_totalSize = report.totalSize;
        m_txtExport = report.txtExport;
        m_csvExport = report.csvExport;
        m_canExport = true;
        m_mhlExport = request.verifyAfterCopy ? report.mhlExport : QString();
        m_canExportMhl = request.verifyAfterCopy && !m_mhlExport.trimmed().isEmpty();
        m_finalStatus = report.finalStatus;
        m_verificationPerformed = report.verificationPerformed;
        emit exportStateChanged();
        emit canExportMhlChanged();
        emit summaryChanged();
    });
    connect(worker, &DitOffloadWorker::failed, this, [this](const QString &message) {
        setBusy(false);
        setOverallProgress(0.0);
        setStatusText(message);
        setPass(false);
        appendLog(QStringLiteral("Offload failed: %1").arg(message), LogSeverity::Error);
    });
    connect(worker, &DitOffloadWorker::cancelled, this, [this] {
        setBusy(false);
        setOverallProgress(0.0);
        setStatusText(QStringLiteral("Offload cancelled."));
        setPass(false);
        appendLog(QStringLiteral("Offload cancelled by user."), LogSeverity::Warn);
    });
    connect(worker, &DitOffloadWorker::finished, thread, [thread](const FinalReportData &) { thread->quit(); });
    connect(worker, &DitOffloadWorker::failed, thread, &QThread::quit);
    connect(worker, &DitOffloadWorker::cancelled, thread, &QThread::quit);
    connect(thread, &QThread::finished, worker, &QObject::deleteLater);
    connect(thread, &QThread::finished, thread, &QObject::deleteLater);
    connect(thread, &QThread::finished, this, [this] { m_activeWorker = nullptr; });
    thread->start();
}

void AppController::cancelOffload()
{
    if (!m_busy || !m_activeWorker) {
        return;
    }
    m_activeWorker->cancel();
    appendLog(QStringLiteral("Cancelling offload..."), LogSeverity::Warn);
}

void AppController::exportTxt()
{
    writeExport(tr("Export TXT Report"), QStringLiteral("seder-dit-report.txt"), m_txtExport);
}

void AppController::exportCsv()
{
    writeExport(tr("Export CSV Report"), QStringLiteral("seder-dit-report.csv"), m_csvExport);
}

void AppController::exportMhl()
{
    if (!canExportMhl()) {
        setStatusText(QStringLiteral("MHL export requires Verify after copy."));
        appendLog(QStringLiteral("MHL export skipped: Verify after copy was not enabled."));
        return;
    }
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

void AppController::appendLog(const QString &line, LogSeverity severity)
{
    QString severityText = QStringLiteral("INFO");
    if (severity == LogSeverity::Warn) severityText = QStringLiteral("WARN");
    if (severity == LogSeverity::Error) severityText = QStringLiteral("ERROR");
    const QString timestamp = QDateTime::currentDateTime().toString(QStringLiteral("HH:mm:ss"));
    m_logLines.append(QStringLiteral("[%1] [%2] %3").arg(timestamp, severityText, line));
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
        appendLog(QStringLiteral("Export skipped because no report is ready."), LogSeverity::Warn);
        return;
    }
    const QString path = QFileDialog::getSaveFileName(nullptr, caption, defaultName);
    if (path.isEmpty()) {
        appendLog(QStringLiteral("Export canceled."), LogSeverity::Warn);
        return;
    }
    QSaveFile file(path);
    if (!file.open(QIODevice::WriteOnly | QIODevice::Text)) {
        setStatusText(QStringLiteral("Unable to write %1").arg(path));
        appendLog(QStringLiteral("Unable to write %1").arg(path), LogSeverity::Error);
        return;
    }
    file.write(contents.toUtf8());
    if (!file.commit()) {
        setStatusText(QStringLiteral("Unable to save %1").arg(path));
        appendLog(QStringLiteral("Unable to save %1").arg(path), LogSeverity::Error);
        return;
    }
    setStatusText(QStringLiteral("Export complete."));
    appendLog(QStringLiteral("Exported %1").arg(path));
}
void AppController::clearLog()
{
    if (m_logLines.isEmpty()) return;
    m_logLines.clear();
    emit logLinesChanged();
    appendLog(QStringLiteral("Log cleared."));
}

void AppController::copyLog()
{
    QClipboard *clipboard = QGuiApplication::clipboard();
    if (!clipboard) {
        appendLog(QStringLiteral("Clipboard is unavailable."), LogSeverity::Warn);
        return;
    }
    clipboard->setText(m_logLines.join(u'\n'));
    appendLog(QStringLiteral("Copied log to clipboard."));
}
