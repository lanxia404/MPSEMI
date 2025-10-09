#include <fcitx/addonfactory.h>
#include <fcitx/addonmanager.h>
#include <fcitx/candidatelist.h>
#include <fcitx/inputmethodengine.h>
#include <fcitx/inputpanel.h>
#include <fcitx/instance.h>
#include <fcitx/text.h>
#include <fcitx-utils/key.h>
#include <fcitx-utils/macros.h>
#include <cstring>
#include <functional>
#include <memory>
#include <string>
#include <vector>
#include <cstdint>

#include "mpsemi_notifier.h"

// ---- Rust C-ABI ----
extern "C"
{
    struct MPSEMI_Cand
    {
        const char *text;
    };
    void *mpsemi_engine_new();
    void mpsemi_engine_free(void *eng);
    // 傳入單字元或UTF-8字串；回傳是否消耗事件
    bool mpsemi_process_utf8(void *eng, const char *s);
    // 取得/釋放字串
    char *mpsemi_preedit(void *eng);
    uint32_t mpsemi_candidate_count(void *eng);
    char *mpsemi_candidate_at(void *eng, uint32_t idx);
    char *mpsemi_commit(void *eng);
    bool mpsemi_adjust_selection(void *eng, int32_t offset);
    void mpsemi_free_cstr(char *s);
}

namespace
{
constexpr const char kMPSEMICurrentVersion[] = "0.1.0";
}

class MPSEMICandidateWord final : public fcitx::CandidateWord
{
public:
    MPSEMICandidateWord(fcitx::Text text,
                        std::function<void(fcitx::InputContext *)> cb)
        : fcitx::CandidateWord(std::move(text)), callback_(std::move(cb))
    {
    }

    void select(fcitx::InputContext *inputContext) const override
    {
        if (callback_)
        {
            callback_(inputContext);
        }
    }

private:
    std::function<void(fcitx::InputContext *)> callback_;
};

class MPSEMIEngine final : public fcitx::InputMethodEngine
{
public:
    explicit MPSEMIEngine(fcitx::Instance *instance)
        : notifier_(instance ? std::make_unique<MPSEMIUpdateNotifier>(
                                    instance, kMPSEMICurrentVersion)
                            : nullptr),
        core_(mpsemi_engine_new())
    {
        setCanRestart(true);
    }

    ~MPSEMIEngine()
    {
        mpsemi_engine_free(core_);
    }

    void keyEvent(const fcitx::InputMethodEntry &,
                fcitx::KeyEvent &key) override
    {
        if (key.isRelease())
        {
            return;
        }

        auto ic = key.inputContext();

        auto sym = key.key().sym();
        if (sym == FcitxKey_Left || sym == FcitxKey_Right ||
            sym == FcitxKey_Up || sym == FcitxKey_Down)
        {
            int offset = (sym == FcitxKey_Left || sym == FcitxKey_Up) ? -1 : 1;
            if (mpsemi_adjust_selection(core_, offset))
            {
                refreshUI(ic);
                key.filterAndAccept();
            }
            return;
        }

        // 將可見字元與空白/Enter轉給 Rust；其他鍵放行
        std::string text;
        if (sym == FcitxKey_space)
            text = " ";
        else if (sym == FcitxKey_Return)
            text = "\n";
        else if (sym == FcitxKey_BackSpace)
            text = "\b";
        else if (sym == FcitxKey_Escape)
            text = "\x1b";
        else if (key.key().isSimple())
            text = key.key().toString();
        else
        {
            return;
        }

        bool consumed = mpsemi_process_utf8(core_, text.c_str());
        if (consumed && ic && (sym == FcitxKey_space || sym == FcitxKey_Return))
        {
            commitToContext(ic);
        }
        else
        {
            refreshUI(ic);
        }

        if (consumed)
            key.filterAndAccept();
    }

    void activate(const fcitx::InputMethodEntry &entry, fcitx::InputContextEvent &event) override
    {
        FCITX_UNUSED(entry);
        refreshUI(event.inputContext());
        if (notifier_)
        {
            notifier_->presentIn(event.inputContext());
        }
    }

    void reset(const fcitx::InputMethodEntry &, fcitx::InputContextEvent &event) override
    {
        mpsemi_process_utf8(core_, "\x1b");
        refreshUI(event.inputContext());
    }

    void deactivate(const fcitx::InputMethodEntry &entry, fcitx::InputContextEvent &event) override
    {
        FCITX_UNUSED(entry);
        if (notifier_)
        {
            notifier_->removeFrom(event.inputContext());
        }
    }

private:
    void commitToContext(fcitx::InputContext *ctx)
    {
        if (!ctx)
        {
            if (char *s = mpsemi_commit(core_))
            {
                mpsemi_free_cstr(s);
            }
            return;
        }

        if (char *s = mpsemi_commit(core_))
        {
            std::string out(s);
            mpsemi_free_cstr(s);
            ctx->commitString(out);
            refreshUI(ctx);
        }
        else
        {
            refreshUI(ctx);
        }
    }

    void refreshUI(fcitx::InputContext *ctx)
    {
        if (!ctx)
        {
            return;
        }

        std::string preeditStr;
        if (char *p = mpsemi_preedit(core_))
        {
            preeditStr.assign(p);
            mpsemi_free_cstr(p);
        }

        fcitx::Text preeditText;
        if (!preeditStr.empty())
        {
            fcitx::TextFormatFlags format = fcitx::TextFormatFlag::Underline;
            format |= fcitx::TextFormatFlag::HighLight;
            preeditText.append(preeditStr, format);
            preeditText.setCursor(preeditText.textLength());
        }

        ctx->inputPanel().setPreedit(preeditText);
        ctx->inputPanel().setClientPreedit(preeditText);
        ctx->updatePreedit();

        uint32_t count = mpsemi_candidate_count(core_);
        if (count <= 1)
        {
            ctx->inputPanel().setCandidateList(nullptr);
            ctx->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
            if (notifier_)
            {
                notifier_->presentIn(ctx);
            }
            return;
        }

        auto list = std::make_unique<fcitx::CommonCandidateList>();
        list->setSelectionKey({
            fcitx::Key(FcitxKey_1), fcitx::Key(FcitxKey_2), fcitx::Key(FcitxKey_3),
            fcitx::Key(FcitxKey_4), fcitx::Key(FcitxKey_5), fcitx::Key(FcitxKey_6),
            fcitx::Key(FcitxKey_7), fcitx::Key(FcitxKey_8), fcitx::Key(FcitxKey_9)
        });

        bool hasCandidate = false;
        for (uint32_t i = 1; i < count; ++i)
        {
            if (char *c = mpsemi_candidate_at(core_, i))
            {
                std::string txt(c);
                mpsemi_free_cstr(c);
                if (txt.empty())
                {
                    continue;
                }
                hasCandidate = true;
                list->append<MPSEMICandidateWord>(
                    fcitx::Text(txt),
                    [this, engineIndex = static_cast<int32_t>(i)](fcitx::InputContext *candidateCtx)
                    {
                        // 將選定項目旋轉到首位後提交
                        mpsemi_adjust_selection(core_, engineIndex);
                        commitToContext(candidateCtx);
                    });
            }
        }

        if (!hasCandidate)
        {
            ctx->inputPanel().setCandidateList(nullptr);
        }
        else
        {
            list->setCursorIndex(0);
            ctx->inputPanel().setCandidateList(std::move(list));
        }
        ctx->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
        if (notifier_)
        {
            notifier_->presentIn(ctx);
        }
    }
    std::unique_ptr<MPSEMIUpdateNotifier> notifier_;
    void *core_;
};

class MPSEMIEngineFactory final : public fcitx::AddonFactory
{
public:
    fcitx::AddonInstance *create(fcitx::AddonManager *manager)
    {
        fcitx::Instance *instance = manager ? manager->instance() : nullptr;
        return new MPSEMIEngine(instance);
    }
};

FCITX_ADDON_FACTORY(MPSEMIEngineFactory)
