import { type FormEvent, useLayoutEffect, useRef, useState } from "react";
import { useI18n } from "../../i18n";
import { normalizeDecimalDigits } from "../../i18n/digits";
import { signInErrors } from "../copy";
import type { ConfirmCodeResult, RequestCodeResult } from "../ports";
import { requestSignInCode, signIn } from "../usecases/account";
import type { Context } from "../usecases/context";
import { StandIn } from "./shared";

type EmailError = Extract<RequestCodeResult, { ok: false }>["error"];
type CodeError = Extract<ConfirmCodeResult, { ok: false }>["error"];
type Step = { readonly at: "email"; readonly email?: string; readonly error?: EmailError } | { readonly at: "code"; readonly email: string; readonly standInCode?: string; readonly error?: CodeError };

/** FR-001: sign in by confirming a code, with no password and one thing to type. */
export function SignInScreen({ ctx, onSignedIn }: { readonly ctx: Context; readonly onSignedIn: () => Promise<void> }) {
  const { t } = useI18n();
  const [step, setStep] = useState<Step>({ at: "email" });
  const heading = useRef<HTMLHeadingElement>(null);
  const codeField = useRef<HTMLInputElement>(null);
  const focusStep = useRef(false);
  const [requesting, setRequesting] = useState(false);
  const errors = signInErrors(t);

  useLayoutEffect(() => {
    if (!focusStep.current) return;
    focusStep.current = false;
    heading.current?.focus();
  }, [step]);

  async function askForCode(email: string) {
    if (requesting) return;
    setRequesting(true);
    try {
      const result = await requestSignInCode(ctx, email);
      if (result.ok && codeField.current !== null) codeField.current.value = "";
      focusStep.current = result.ok || step.at !== "email";
      setStep(result.ok ? { at: "code", email, standInCode: result.standInCode } : { at: "email", email, error: result.error });
    } finally {
      setRequesting(false);
    }
  }

  function onEmail(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const email = String(new FormData(event.currentTarget).get("email") ?? "");
    void askForCode(email);
  }

  async function onCode(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (step.at !== "code") return;
    const code = normalizeDecimalDigits(String(new FormData(event.currentTarget).get("code") ?? ""));
    const result = await signIn(ctx, step.email, code);
    if (result.ok) await onSignedIn();
    else setStep({ ...step, error: result.error });
  }

  if (step.at === "email") {
    return (
      <form onSubmit={(e) => void onEmail(e)}>
        <h2 tabIndex={-1} ref={heading}>{t("Sign in to try Agely")}</h2>
        <p className="lead">{t("Tell us where to send a sign-in code. There is no password and nothing else to fill in.")}</p>
        <div className="field">
          <label htmlFor="product-email">{t("Your email address")}</label>
          <input id="product-email" name="email" type="text" inputMode="email" autoComplete="email" defaultValue={step.email} required />
        </div>
        {step.error !== undefined && (
          <p className="notice notice--error" role="alert">
            {errors[step.error]}
          </p>
        )}
        <div className="actions">
          <button type="submit" className="primary" disabled={requesting}>
            {t("Send me a code")}
          </button>
        </div>
      </form>
    );
  }

  const [beforeCode, afterCode] = t("Your code is {0}").split("{0}");
  return (
    <form onSubmit={(e) => void onCode(e)}>
      <h2 tabIndex={-1} ref={heading}>{t("Enter your code")}</h2>
      <p className="lead">{t("We made a six-digit code for {0}.", step.email)}</p>
      {step.standInCode !== undefined && (
        <StandIn ctx={ctx} service="sign-in">
          <p className="stand-in__code">
            {beforeCode}<strong>{step.standInCode}</strong>{afterCode}
          </p>
        </StandIn>
      )}
      <div className="field">
        <label htmlFor="product-code">{t("Six-digit code")}</label>
        <input id="product-code" name="code" type="text" inputMode="numeric" autoComplete="one-time-code" ref={codeField} required />
      </div>
      {step.error !== undefined && (
        <p className="notice notice--error" role="alert">
          {errors[step.error]}
        </p>
      )}
      <div className="actions">
        <button type="submit" className="primary" disabled={requesting}>
          {t("Sign in")}
        </button>
        {(step.error === "expired" || step.error === "used") && (
          <button type="button" className="secondary" disabled={requesting} onClick={() => void askForCode(step.email)}>
            {t("Send me a code")}
          </button>
        )}
        <button type="button" className="quiet" disabled={requesting} onClick={() => {
          focusStep.current = true;
          setStep({ at: "email", email: step.email });
        }}>
          {t("Use a different address")}
        </button>
      </div>
    </form>
  );
}
