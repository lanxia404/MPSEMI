#pragma once

#include <memory>
#include <string>

namespace fcitx
{
class Instance;
class InputContext;
class SimpleAction;
} // namespace fcitx

class MPSEMIUpdateNotifier
{
public:
    MPSEMIUpdateNotifier(fcitx::Instance *instance, std::string currentVersion);
    ~MPSEMIUpdateNotifier();

    bool updateAvailable() const;
    void presentIn(fcitx::InputContext *ctx);
    void removeFrom(fcitx::InputContext *ctx);

private:
    void initialize();
    bool recordVersion();
    void setupRestartAction();
    void notifyUpdate();

    fcitx::Instance *instance_ = nullptr;
    std::string version_;
    bool updateAvailable_ = false;
    std::unique_ptr<fcitx::SimpleAction> restartAction_;
};

