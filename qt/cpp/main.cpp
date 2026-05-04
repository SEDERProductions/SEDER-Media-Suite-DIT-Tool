#include "AppController.h"
#include "DestinationListModel.h"
#include "ThemeController.h"

#include <QApplication>
#include <QQmlApplicationEngine>
#include <QQmlContext>

int main(int argc, char *argv[])
{
    QApplication app(argc, argv);
    app.setOrganizationName(QStringLiteral("Seder Productions"));
    app.setApplicationName(QStringLiteral("SEDER Media Suite DIT"));

    ThemeController themeController;
    AppController appController;

    QQmlApplicationEngine engine;
    engine.rootContext()->setContextProperty(QStringLiteral("appController"), &appController);
    engine.rootContext()->setContextProperty(QStringLiteral("themeController"), &themeController);

    QObject::connect(
        &engine,
        &QQmlApplicationEngine::objectCreationFailed,
        &app,
        [] { QCoreApplication::exit(-1); },
        Qt::QueuedConnection);
    engine.loadFromModule(QStringLiteral("SederDit"), QStringLiteral("Main"));

    return app.exec();
}
