#pragma once

#include "DestinationListModel.h"
#include "DitOffloadWorker.h"
#include "ThemeController.h"

#include <QObject>
#include <QStringList>

class AppController final : public QObject {
    Q_OBJECT
    Q_PROPERTY(QString sourcePath READ sourcePath WRITE setSourcePath NOTIFY sourcePathChanged)
    Q_PROPERTY(DestinationListModel *destinationModel READ destinationModel CONSTANT)
    Q_PROPERTY(QString projectName READ projectName WRITE setProjectName NOTIFY projectNameChanged)
    Q_PROPERTY(QString shootDate READ shootDate WRITE setShootDate NOTIFY shootDateChanged)
    Q_PROPERTY(QString cardName READ cardName WRITE setCardName NOTIFY cardNameChanged)
    Q_PROPERTY(QString cameraId READ cameraId WRITE setCameraId NOTIFY cameraIdChanged)
    Q_PROPERTY(QString ignorePatterns READ ignorePatterns WRITE setIgnorePatterns NOTIFY ignorePatternsChanged)
    Q_PROPERTY(bool ignoreHiddenSystem READ ignoreHiddenSystem WRITE setIgnoreHiddenSystem NOTIFY ignoreHiddenSystemChanged)
    Q_PROPERTY(bool verifyAfterCopy READ verifyAfterCopy WRITE setVerifyAfterCopy NOTIFY verifyAfterCopyChanged)
    Q_PROPERTY(bool busy READ busy NOTIFY busyChanged)
    Q_PROPERTY(double overallProgress READ overallProgress NOTIFY overallProgressChanged)
    Q_PROPERTY(QString statusText READ statusText NOTIFY statusTextChanged)
    Q_PROPERTY(QString currentFile READ currentFile NOTIFY currentFileChanged)
    Q_PROPERTY(QStringList logLines READ logLines NOTIFY logLinesChanged)
    Q_PROPERTY(bool canExport READ canExport NOTIFY exportStateChanged)
    Q_PROPERTY(quint64 totalFiles READ totalFiles NOTIFY summaryChanged)
    Q_PROPERTY(quint64 totalSize READ totalSize NOTIFY summaryChanged)
    Q_PROPERTY(bool pass READ pass NOTIFY summaryChanged)

public:
    explicit AppController(QObject *parent = nullptr);

    QString sourcePath() const;
    void setSourcePath(const QString &value);
    DestinationListModel *destinationModel() const;
    QString projectName() const;
    void setProjectName(const QString &value);
    QString shootDate() const;
    void setShootDate(const QString &value);
    QString cardName() const;
    void setCardName(const QString &value);
    QString cameraId() const;
    void setCameraId(const QString &value);
    QString ignorePatterns() const;
    void setIgnorePatterns(const QString &value);
    bool ignoreHiddenSystem() const;
    void setIgnoreHiddenSystem(bool value);
    bool verifyAfterCopy() const;
    void setVerifyAfterCopy(bool value);
    bool busy() const;
    double overallProgress() const;
    QString statusText() const;
    QString currentFile() const;
    QStringList logLines() const;
    bool canExport() const;
    quint64 totalFiles() const;
    quint64 totalSize() const;
    bool pass() const;

    Q_INVOKABLE void chooseSourceFolder();
    Q_INVOKABLE void addDestinationFolder();
    Q_INVOKABLE void removeDestination(int index);
    Q_INVOKABLE void startOffload();
    Q_INVOKABLE void cancelOffload();
    Q_INVOKABLE void exportTxt();
    Q_INVOKABLE void exportCsv();
    Q_INVOKABLE void exportMhl();
    Q_INVOKABLE QString formatBytes(quint64 value) const;

signals:
    void sourcePathChanged();
    void projectNameChanged();
    void shootDateChanged();
    void cardNameChanged();
    void cameraIdChanged();
    void ignorePatternsChanged();
    void ignoreHiddenSystemChanged();
    void verifyAfterCopyChanged();
    void busyChanged();
    void overallProgressChanged();
    void statusTextChanged();
    void currentFileChanged();
    void logLinesChanged();
    void exportStateChanged();
    void summaryChanged();

private:
    void appendLog(const QString &line);
    void setBusy(bool value);
    void setOverallProgress(double value);
    void setStatusText(const QString &value);
    void setCurrentFile(const QString &value);
    void setPass(bool value);
    void writeExport(const QString &caption, const QString &defaultName, const QString &contents);

    DestinationListModel *m_destinationModel = nullptr;
    QString m_sourcePath;
    QString m_projectName;
    QString m_shootDate;
    QString m_cardName;
    QString m_cameraId;
    QString m_ignorePatterns = QStringLiteral(".DS_Store, Thumbs.db, desktop.ini, .Spotlight-V100, .Trashes");
    bool m_ignoreHiddenSystem = true;
    bool m_verifyAfterCopy = true;
    bool m_busy = false;
    double m_overallProgress = 0.0;
    QString m_statusText = QStringLiteral("Ready for offload.");
    QString m_currentFile;
    QStringList m_logLines;
    bool m_canExport = false;
    quint64 m_totalFiles = 0;
    quint64 m_totalSize = 0;
    bool m_pass = false;
    DitOffloadWorker *m_activeWorker = nullptr;
    QString m_txtExport;
    QString m_csvExport;
    QString m_mhlExport;
};
