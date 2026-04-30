#pragma once

#include "DitFilterProxyModel.h"
#include "DitResult.h"
#include "DitResultModel.h"

#include <QObject>
#include <QStringList>

class AppController final : public QObject {
    Q_OBJECT
    Q_PROPERTY(QString sourcePath READ sourcePath WRITE setSourcePath NOTIFY sourcePathChanged)
    Q_PROPERTY(QString destinationPath READ destinationPath WRITE setDestinationPath NOTIFY destinationPathChanged)
    Q_PROPERTY(QString projectName READ projectName WRITE setProjectName NOTIFY projectNameChanged)
    Q_PROPERTY(QString shootDate READ shootDate WRITE setShootDate NOTIFY shootDateChanged)
    Q_PROPERTY(QString cardName READ cardName WRITE setCardName NOTIFY cardNameChanged)
    Q_PROPERTY(QString cameraId READ cameraId WRITE setCameraId NOTIFY cameraIdChanged)
    Q_PROPERTY(QString ignorePatterns READ ignorePatterns WRITE setIgnorePatterns NOTIFY ignorePatternsChanged)
    Q_PROPERTY(int compareMode READ compareMode WRITE setCompareMode NOTIFY compareModeChanged)
    Q_PROPERTY(bool ignoreHiddenSystem READ ignoreHiddenSystem WRITE setIgnoreHiddenSystem NOTIFY ignoreHiddenSystemChanged)
    Q_PROPERTY(bool busy READ busy NOTIFY busyChanged)
    Q_PROPERTY(double progress READ progress NOTIFY progressChanged)
    Q_PROPERTY(QString statusText READ statusText NOTIFY statusTextChanged)
    Q_PROPERTY(QStringList logLines READ logLines NOTIFY logLinesChanged)
    Q_PROPERTY(bool canExport READ canExport NOTIFY exportStateChanged)
    Q_PROPERTY(bool mhlAvailable READ mhlAvailable NOTIFY exportStateChanged)
    Q_PROPERTY(quint64 onlyACount READ onlyACount NOTIFY summaryChanged)
    Q_PROPERTY(quint64 onlyBCount READ onlyBCount NOTIFY summaryChanged)
    Q_PROPERTY(quint64 changedCount READ changedCount NOTIFY summaryChanged)
    Q_PROPERTY(quint64 matchingCount READ matchingCount NOTIFY summaryChanged)
    Q_PROPERTY(quint64 totalFiles READ totalFiles NOTIFY summaryChanged)
    Q_PROPERTY(quint64 totalFolders READ totalFolders NOTIFY summaryChanged)
    Q_PROPERTY(quint64 totalSize READ totalSize NOTIFY summaryChanged)
    Q_PROPERTY(bool pass READ pass NOTIFY summaryChanged)

public:
    AppController(DitResultModel *resultModel, DitFilterProxyModel *filterModel, QObject *parent = nullptr);

    QString sourcePath() const;
    void setSourcePath(const QString &value);
    QString destinationPath() const;
    void setDestinationPath(const QString &value);
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
    int compareMode() const;
    void setCompareMode(int value);
    bool ignoreHiddenSystem() const;
    void setIgnoreHiddenSystem(bool value);
    bool busy() const;
    double progress() const;
    QString statusText() const;
    QStringList logLines() const;
    bool canExport() const;
    bool mhlAvailable() const;

    quint64 onlyACount() const;
    quint64 onlyBCount() const;
    quint64 changedCount() const;
    quint64 matchingCount() const;
    quint64 totalFiles() const;
    quint64 totalFolders() const;
    quint64 totalSize() const;
    bool pass() const;

    Q_INVOKABLE void chooseSourceFolder();
    Q_INVOKABLE void chooseDestinationFolder();
    Q_INVOKABLE void startComparison();
    Q_INVOKABLE void setFilter(int filter);
    Q_INVOKABLE void exportTxt();
    Q_INVOKABLE void exportCsv();
    Q_INVOKABLE void exportMhl();
    Q_INVOKABLE QString formatBytes(quint64 value) const;

signals:
    void sourcePathChanged();
    void destinationPathChanged();
    void projectNameChanged();
    void shootDateChanged();
    void cardNameChanged();
    void cameraIdChanged();
    void ignorePatternsChanged();
    void compareModeChanged();
    void ignoreHiddenSystemChanged();
    void busyChanged();
    void progressChanged();
    void statusTextChanged();
    void logLinesChanged();
    void exportStateChanged();
    void summaryChanged();

private:
    void appendLog(const QString &line);
    void setBusy(bool value);
    void setProgress(double value);
    void setStatusText(const QString &value);
    void setResult(const DitResult &result);
    void resetResult();
    void writeExport(const QString &caption, const QString &defaultName, const QString &contents);

    DitResultModel *m_resultModel = nullptr;
    DitFilterProxyModel *m_filterModel = nullptr;
    QString m_sourcePath;
    QString m_destinationPath;
    QString m_projectName;
    QString m_shootDate;
    QString m_cardName;
    QString m_cameraId;
    QString m_ignorePatterns = QStringLiteral(".DS_Store, Thumbs.db, desktop.ini, .Spotlight-V100, .Trashes");
    int m_compareMode = 2;
    bool m_ignoreHiddenSystem = true;
    bool m_busy = false;
    double m_progress = 0.0;
    QString m_statusText = QStringLiteral("Ready for card verification.");
    QStringList m_logLines;
    DitSummaryData m_summary;
    QString m_txtExport;
    QString m_csvExport;
    QString m_mhlExport;
};
