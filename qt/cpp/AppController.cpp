#include "AppController.h"

#include "DitCompareWorker.h"

#include <QFileDialog>
#include <QSaveFile>
#include <QThread>
#include <QtGlobal>
#include <functional>

namespace {

void setIfChanged(QString &field, const QString &value, const std::function<void()> &notify)
{
    if (field == value) {
        return;
    }
    field = value;
    notify();
}

} // namespace

AppController::AppController(DitResultModel *resultModel, DitFilterProxyModel *filterModel, QObject *parent)
    : QObject(parent)
    , m_resultModel(resultModel)
    , m_filterModel(filterModel)
{
    appendLog(QStringLiteral("Ready to verify source and destination folders."));
}

QString AppController::sourcePath() const { return m_sourcePath; }
void AppController::setSourcePath(const QString &value) { setIfChanged(m_sourcePath, value, [this] { emit sourcePathChanged(); }); }
QString AppController::destinationPath() const { return m_destinationPath; }
void AppController::setDestinationPath(const QString &value) { setIfChanged(m_destinationPath, value, [this] { emit destinationPathChanged(); }); }
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
int AppController::compareMode() const { return m_compareMode; }

void AppController::setCompareMode(int value)
{
    if (m_compareMode == value) {
        return;
    }
    m_compareMode = value;
    emit compareModeChanged();
}

bool AppController::ignoreHiddenSystem() const { return m_ignoreHiddenSystem; }

void AppController::setIgnoreHiddenSystem(bool value)
{
    if (m_ignoreHiddenSystem == value) {
        return;
    }
    m_ignoreHiddenSystem = value;
    emit ignoreHiddenSystemChanged();
}

bool AppController::busy() const { return m_busy; }
double AppController::progress() const { return m_progress; }
QString AppController::statusText() const { return m_statusText; }
QStringList AppController::logLines() const { return m_logLines; }
bool AppController::canExport() const { return !m_txtExport.isEmpty() && !m_csvExport.isEmpty(); }
bool AppController::mhlAvailable() const { return !m_mhlExport.isEmpty() && m_summary.mhlAvailable; }
quint64 AppController::onlyACount() const { return m_summary.onlyA; }
quint64 AppController::onlyBCount() const { return m_summary.onlyB; }
quint64 AppController::changedCount() const { return m_summary.changed; }
quint64 AppController::matchingCount() const { return m_summary.matching; }
quint64 AppController::totalFiles() const { return m_summary.totalFiles; }
quint64 AppController::totalFolders() const { return m_summary.totalFolders; }
quint64 AppController::totalSize() const { return m_summary.totalSize; }
bool AppController::pass() const { return m_summary.pass; }

void AppController::chooseSourceFolder()
{
    const QString path = QFileDialog::getExistingDirectory(nullptr, tr("Choose Source Folder"), m_sourcePath);
    if (!path.isEmpty()) {
        setSourcePath(path);
    }
}

void AppController::chooseDestinationFolder()
{
    const QString path = QFileDialog::getExistingDirectory(nullptr, tr("Choose Destination Folder"), m_destinationPath);
    if (!path.isEmpty()) {
        setDestinationPath(path);
    }
}

void AppController::startComparison()
{
    if (m_busy) {
        return;
    }
    if (m_sourcePath.isEmpty()) {
        setStatusText(QStringLiteral("Missing source folder."));
        appendLog(QStringLiteral("Source folder is required."));
        return;
    }
    if (m_destinationPath.isEmpty()) {
        setStatusText(QStringLiteral("Missing destination folder."));
        appendLog(QStringLiteral("Destination folder is required."));
        return;
    }

    resetResult();
    setBusy(true);
    setProgress(0.0);
    setStatusText(QStringLiteral("Preparing verification..."));
    appendLog(QStringLiteral("Starting DIT verification."));

    DitRequestData request;
    request.sourcePath = m_sourcePath;
    request.destinationPath = m_destinationPath;
    request.projectName = m_projectName;
    request.shootDate = m_shootDate;
    request.cardName = m_cardName;
    request.cameraId = m_cameraId;
    request.ignorePatterns = m_ignorePatterns;
    request.compareMode = m_compareMode;
    request.ignoreHiddenSystem = m_ignoreHiddenSystem;

    auto *thread = new QThread(this);
    auto *worker = new DitCompareWorker(request);
    worker->moveToThread(thread);

    connect(thread, &QThread::started, worker, &DitCompareWorker::run);
    connect(worker, &DitCompareWorker::progress, this, [this](const DitProgress &update) {
        setStatusText(update.status);
        if (update.phase == QStringLiteral("complete")) {
            setProgress(1.0);
        } else if (update.processedFiles > 0) {
            setProgress(qMin(0.95, m_progress + 0.01));
        }
    });
    connect(worker, &DitCompareWorker::finished, this, [this](const DitResult &result) {
        setResult(result);
        setBusy(false);
        setProgress(1.0);
        setStatusText(result.summary.pass ? QStringLiteral("Verification complete: PASS.") : QStringLiteral("Verification complete: FAIL."));
        appendLog(QStringLiteral("Verification complete. Rows: %1.").arg(result.rows.size()));
    });
    connect(worker, &DitCompareWorker::failed, this, [this](const QString &message) {
        setBusy(false);
        setProgress(0.0);
        setStatusText(message);
        appendLog(QStringLiteral("Verification failed: %1").arg(message));
    });
    connect(worker, &DitCompareWorker::finished, thread, &QThread::quit);
    connect(worker, &DitCompareWorker::failed, thread, &QThread::quit);
    connect(thread, &QThread::finished, worker, &QObject::deleteLater);
    connect(thread, &QThread::finished, thread, &QObject::deleteLater);
    thread->start();
}

void AppController::setFilter(int filter)
{
    if (m_filterModel) {
        m_filterModel->setFilter(filter);
    }
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
    if (!mhlAvailable()) {
        setStatusText(QStringLiteral("MHL export requires checksum mode."));
        appendLog(QStringLiteral("MHL export skipped because the current report is not checksum-backed."));
        return;
    }
    writeExport(tr("Export MHL Report"), QStringLiteral("seder-dit-report.mhl"), m_mhlExport);
}

QString AppController::formatBytes(quint64 value) const
{
    return DitResultModel::formatBytes(value);
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
    if (m_busy == value) {
        return;
    }
    m_busy = value;
    emit busyChanged();
}

void AppController::setProgress(double value)
{
    value = qBound(0.0, value, 1.0);
    if (qFuzzyCompare(m_progress, value)) {
        return;
    }
    m_progress = value;
    emit progressChanged();
}

void AppController::setStatusText(const QString &value)
{
    if (m_statusText == value) {
        return;
    }
    m_statusText = value;
    emit statusTextChanged();
}

void AppController::setResult(const DitResult &result)
{
    if (m_resultModel) {
        m_resultModel->setRows(result.rows);
    }
    m_summary = result.summary;
    m_txtExport = result.txtExport;
    m_csvExport = result.csvExport;
    m_mhlExport = result.mhlExport;
    emit summaryChanged();
    emit exportStateChanged();
}

void AppController::resetResult()
{
    if (m_resultModel) {
        m_resultModel->clear();
    }
    m_summary = {};
    m_txtExport.clear();
    m_csvExport.clear();
    m_mhlExport.clear();
    emit summaryChanged();
    emit exportStateChanged();
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
        const QString message = QStringLiteral("Unable to write %1").arg(path);
        setStatusText(message);
        appendLog(message);
        return;
    }
    file.write(contents.toUtf8());
    if (!file.commit()) {
        const QString message = QStringLiteral("Unable to save %1").arg(path);
        setStatusText(message);
        appendLog(message);
        return;
    }
    setStatusText(QStringLiteral("Export complete."));
    appendLog(QStringLiteral("Exported %1").arg(path));
}
