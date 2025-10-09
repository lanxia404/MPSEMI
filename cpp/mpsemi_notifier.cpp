#include "mpsemi_notifier.h"

#include <fcitx/addoninstance.h>
#include <fcitx/addonmanager.h>
#include <fcitx/action.h>
#include <fcitx/inputcontext.h>
#include <fcitx/instance.h>
#include <fcitx/statusarea.h>
#include <fcitx/userinterface.h>
#include <fcitx/userinterfacemanager.h>
#include <fcitx-utils/key.h>
#include <fcitx-utils/log.h>
#include <fcitx-utils/macros.h>
#include <fcitx-utils/standardpaths.h>

#include <algorithm>
#include <filesystem>
#include <fstream>
#include <functional>
#include <vector>

namespace
{
constexpr const char kRestartActionName[] = "mpsemi-restart-fcitx5";
constexpr const char kNotificationAppName[] = "MPSEMI";
constexpr const char kNotificationIcon[] = "input-keyboard";
}

namespace fcitx
{
using NotificationActionCallback = std::function<void(const std::string &)>;
using NotificationClosedCallback = std::function<void(uint32_t)>;
} // namespace fcitx

FCITX_ADDON_DECLARE_FUNCTION(
    Notifications, sendNotification,
    uint32_t(const std::string &appName, uint32_t replaceId,
            const std::string &appIcon, const std::string &summary,
            const std::string &body, const std::vector<std::string> &actions,
            int32_t timeout, fcitx::NotificationActionCallback actionCallback,
            fcitx::NotificationClosedCallback closedCallback));

MPSEMIUpdateNotifier::MPSEMIUpdateNotifier(fcitx::Instance *instance,
                                        std::string currentVersion)
    : instance_(instance), version_(std::move(currentVersion))
{
    initialize();
}

MPSEMIUpdateNotifier::~MPSEMIUpdateNotifier()
{
    if (instance_ && restartAction_)
    {
        instance_->userInterfaceManager().unregisterAction(restartAction_.get());
    }
}

bool MPSEMIUpdateNotifier::updateAvailable() const
{
    return updateAvailable_;
}

void MPSEMIUpdateNotifier::presentIn(fcitx::InputContext *ctx)
{
    if (!ctx || !updateAvailable_ || !restartAction_)
    {
        return;
    }

    auto actions =
        ctx->statusArea().actions(fcitx::StatusGroup::AfterInputMethod);
    if (std::find(actions.begin(), actions.end(), restartAction_.get()) ==
        actions.end())
    {
        ctx->statusArea().addAction(fcitx::StatusGroup::AfterInputMethod,
                                    restartAction_.get());
        ctx->updateUserInterface(fcitx::UserInterfaceComponent::StatusArea);
    }
}

void MPSEMIUpdateNotifier::removeFrom(fcitx::InputContext *ctx)
{
    if (!ctx || !restartAction_)
    {
        return;
    }

    ctx->statusArea().removeAction(restartAction_.get());
    ctx->updateUserInterface(fcitx::UserInterfaceComponent::StatusArea);
}

void MPSEMIUpdateNotifier::initialize()
{
    if (!instance_)
    {
        return;
    }

    updateAvailable_ = recordVersion();
    if (updateAvailable_)
    {
        setupRestartAction();
        notifyUpdate();
    }
}

bool MPSEMIUpdateNotifier::recordVersion()
{
    namespace fs = std::filesystem;
    const auto &paths = fcitx::StandardPaths::global();
    fs::path base =
        paths.userDirectory(fcitx::StandardPathsType::Data) / "fcitx5" /
        "mpsemi";
    std::error_code ec;
    fs::create_directories(base, ec);

    const fs::path versionPath = base / "version";
    std::string storedVersion;
    if (std::ifstream in(versionPath); in.is_open())
    {
        std::getline(in, storedVersion);
    }

    if (storedVersion == version_)
    {
        return false;
    }

    if (std::ofstream out(versionPath, std::ios::trunc); out.is_open())
    {
        out << version_;
    }
    else
    {
        FCITX_WARN() << "MPSEMI: 無法寫入版本資訊 " << versionPath;
    }
    return true;
}

void MPSEMIUpdateNotifier::setupRestartAction()
{
    if (!instance_ || restartAction_)
    {
        return;
    }

    restartAction_ = std::make_unique<fcitx::SimpleAction>();
    restartAction_->setShortText("重新啟動 Fcitx5");
    restartAction_->setLongText("MPSEMI 更新完成，重新啟動以載入最新版本。");
    restartAction_->setIcon("system-reboot");
    restartAction_->connect<fcitx::SimpleAction::Activated>(
        [this](fcitx::InputContext *)
        {
            if (instance_ && instance_->canRestart())
            {
                instance_->restart();
            }
        });
    instance_->userInterfaceManager().registerAction(kRestartActionName,
                                                     restartAction_.get());
}

void MPSEMIUpdateNotifier::notifyUpdate()
{
    if (!instance_)
    {
        return;
    }

    auto &manager = instance_->addonManager();
    if (auto *addon = manager.addon("notifications", true))
    {
        std::vector<std::string> actions = {"restart", "重新啟動 Fcitx5"};
        addon->call<fcitx::INotifications::sendNotification>(
            kNotificationAppName, 0, kNotificationIcon, "MPSEMI 已更新",
            "若要套用最新功能，請重新啟動 Fcitx5。", actions, -1,
            [this](const std::string &action)
            {
                if (action == "restart" && instance_ && instance_->canRestart())
                {
                    instance_->restart();
                }
            },
            [](uint32_t) {});
    }
    else
    {
        FCITX_INFO() << "MPSEMI: 通知模組未啟用，無法提示更新。";
    }
}

