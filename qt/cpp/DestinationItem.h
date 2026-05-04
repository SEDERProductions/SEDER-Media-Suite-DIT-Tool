#pragma once

#include <QObject>
#include <QString>
#include <QtGlobal>

class DestinationItem : public QObject {
    Q_OBJECT
    Q_PROPERTY(QString path READ path WRITE setPath NOTIFY pathChanged)
    Q_PROPERTY(QString label READ label WRITE setLabel NOTIFY labelChanged)
    Q_PROPERTY(int state READ state NOTIFY stateChanged)
    Q_PROPERTY(double progress READ progress NOTIFY progressChanged)
    Q_PROPERTY(QString error READ error NOTIFY errorChanged)

public:
    enum State {
        Pending = 0,
        Scanning = 1,
        Copying = 2,
        Verifying = 3,
        Complete = 4,
        Failed = 5
    };
    Q_ENUM(State)

    explicit DestinationItem(QObject *parent = nullptr);

    QString path() const;
    void setPath(const QString &value);
    QString label() const;
    void setLabel(const QString &value);
    int state() const;
    void setState(int value);
    double progress() const;
    void setProgress(double value);
    QString error() const;
    void setError(const QString &value);

signals:
    void pathChanged();
    void labelChanged();
    void stateChanged();
    void progressChanged();
    void errorChanged();

private:
    QString m_path;
    QString m_label;
    int m_state = Pending;
    double m_progress = 0.0;
    QString m_error;
};
