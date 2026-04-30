#include "AppController.h"
#include "DitFilterProxyModel.h"
#include "DitResult.h"
#include "DitResultModel.h"
#include "ThemeController.h"

#include <QApplication>
#include <QQmlApplicationEngine>
#include <QQmlContext>

int main(int argc, char *argv[])
{
    QApplication app(argc, argv);
    app.setOrganizationName(QStringLiteral("Seder Productions"));
    app.setApplicationName(QStringLiteral("SEDER Media Suite DIT"));

    qRegisterMetaType<DitProgress>("DitProgress");
    qRegisterMetaType<DitResult>("DitResult");

    DitResultModel resultModel;
    DitFilterProxyModel filterModel;
    filterModel.setSourceModel(&resultModel);

    ThemeController themeController;
    AppController appController(&resultModel, &filterModel);

    QQmlApplicationEngine engine;
    engine.rootContext()->setContextProperty(QStringLiteral("appController"), &appController);
    engine.rootContext()->setContextProperty(QStringLiteral("themeController"), &themeController);
    engine.rootContext()->setContextProperty(QStringLiteral("resultModel"), &resultModel);
    engine.rootContext()->setContextProperty(QStringLiteral("filterModel"), &filterModel);

    QObject::connect(
        &engine,
        &QQmlApplicationEngine::objectCreationFailed,
        &app,
        [] { QCoreApplication::exit(-1); },
        Qt::QueuedConnection);
    engine.loadFromModule(QStringLiteral("SederDit"), QStringLiteral("Main"));

    return app.exec();
}
