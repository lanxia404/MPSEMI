#include <fcitx/addonfactory.h>
#include <fcitx/inputmethodengine.h>
#include <fcitx/inputpanel.h>
#include <fcitx/candidatelist.h>
#include <fcitx/text.h>
#include <fcitx/instance.h>
#include <memory>
#include <string>
#include <vector>
#include <cstring>

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
    void mpsemi_free_cstr(char *s);
}

class MPSEMIEngine final : public fcitx::InputMethodEngine
{
public:
    MPSEMIEngine() : core_(mpsemi_engine_new()) {}
    ~MPSEMIEngine() { mpsemi_engine_free(core_); }

    void keyEvent(const fcitx::InputMethodEntry &,
                  fcitx::KeyEvent &key) override
    {
        if (key.isRelease())
        {
            return;
        }

        // 將可見字元與空白/Enter轉給 Rust；其他鍵放行
        std::string text;
        auto sym = key.key().sym();
        if (sym == FcitxKey_space)
            text = " ";
        else if (sym == FcitxKey_Return)
            text = "\n";
        else if (key.key().isSimple())
            text = key.key().toString();
        else
        {
            return;
        }

        bool consumed = mpsemi_process_utf8(core_, text.c_str());

        // 更新 preedit
        if (auto p = mpsemi_preedit(core_))
        {
            fcitx::Text pre;
            pre.append(std::string(p));
            mpsemi_free_cstr(p);
            auto ic = key.inputContext();
            ic->inputPanel().setPreedit(pre);
        }

        // 更新候選（簡化：只有一筆）
        auto ic = key.inputContext();
        auto list = std::make_unique<fcitx::CommonCandidateList>();
        uint32_t n = mpsemi_candidate_count(core_);
        for (uint32_t i = 0; i < n; ++i)
        {
            if (char *c = mpsemi_candidate_at(core_, i))
            {
                std::string txt(c);
                mpsemi_free_cstr(c);
                list->append<fcitx::CandidateWord>(
                    fcitx::Text(txt),
                    [this](const fcitx::InputMethodEntry &, fcitx::InputContext *ctx)
                    {
                        if (char *s = mpsemi_commit(core_))
                        {
                            std::string out(s);
                            mpsemi_free_cstr(s);
                            ctx->commitString(out);
                            ctx->inputPanel().reset();
                            ctx->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
                        }
                    });
            }
        }
        ic->inputPanel().setCandidateList(std::move(list));
        ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);

        if (consumed)
            key.filterAndAccept();
    }

    void reset(const fcitx::InputMethodEntry &, fcitx::InputContextEvent &) override
    {
        // 簡化：由 mpsemi_process_utf8("") 觸發清空也可
    }

private:
    void *core_;
};

class MPSEMIEngineFactory final : public fcitx::AddonFactory
{
public:
    fcitx::AddonInstance *create(fcitx::AddonManager *)
    {
        return new MPSEMIEngine();
    }
};

FCITX_ADDON_FACTORY(MPSEMIEngineFactory)
