#pragma once

#include <QObject>

class ThemeController final : public QObject {
    Q_OBJECT
    Q_PROPERTY(QString preference READ preference WRITE setPreference NOTIFY preferenceChanged)
    Q_PROPERTY(bool dark READ dark NOTIFY darkChanged)

public:
    explicit ThemeController(QObject *parent = nullptr);

    QString preference() const;
    void setPreference(const QString &preference);
    bool dark() const;

signals:
    void preferenceChanged();
    void darkChanged();

private:
    void load();
    void save();
    void updateDark();

    QString m_preference = QStringLiteral("system");
    bool m_dark = false;
};
