#include "DestinationItem.h"

DestinationItem::DestinationItem(QObject *parent)
    : QObject(parent)
{
}

QString DestinationItem::path() const { return m_path; }
void DestinationItem::setPath(const QString &value) {
    if (m_path != value) { m_path = value; emit pathChanged(); }
}

QString DestinationItem::label() const { return m_label; }
void DestinationItem::setLabel(const QString &value) {
    if (m_label != value) { m_label = value; emit labelChanged(); }
}

int DestinationItem::state() const { return m_state; }
void DestinationItem::setState(int value) {
    if (m_state != value) { m_state = value; emit stateChanged(); }
}

double DestinationItem::progress() const { return m_progress; }
void DestinationItem::setProgress(double value) {
    if (qFuzzyCompare(m_progress, value)) return;
    m_progress = value;
    emit progressChanged();
}

QString DestinationItem::error() const { return m_error; }
void DestinationItem::setError(const QString &value) {
    if (m_error != value) { m_error = value; emit errorChanged(); }
}
