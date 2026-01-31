import React, { useState } from "react";
import type { Theme } from "@/lib/types";

interface FirstRunWizardProps {
  isOpen: boolean;
  onComplete: () => void;
}

export const FirstRunWizard: React.FC<FirstRunWizardProps> = ({ isOpen, onComplete }) => {
  const [step, setStep] = useState(1);
  const [theme, setTheme] = useState<Theme>("system");

  if (!isOpen) return null;

  const handleNext = () => {
    if (step < 3) {
      setStep(step + 1);
    } else {
      onComplete();
    }
  };

  const handleBack = () => {
    if (step > 1) {
      setStep(step - 1);
    }
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
      role="dialog"
      aria-modal="true"
      aria-label="Welcome"
    >
      <div className="flex h-[380px] w-[480px] flex-col rounded-lg bg-white shadow-xl">
        {/* Progress */}
        <div className="flex gap-1 px-6 pt-4">
          {[1, 2, 3].map((s) => (
            <div
              key={s}
              className={`h-1 flex-1 rounded ${
                s <= step ? "bg-blue-500" : "bg-gray-200"
              }`}
            />
          ))}
        </div>

        {/* Content */}
        <div className="flex flex-1 flex-col items-center justify-center px-8 text-center">
          {step === 1 && (
            <>
              <h2 className="text-2xl font-bold text-gray-900">Welcome to MuttonText</h2>
              <p className="mt-3 text-sm text-gray-600">
                MuttonText is a text expansion tool that saves you time by replacing short
                keywords with longer snippets of text. Let us get you set up.
              </p>
            </>
          )}

          {step === 2 && (
            <>
              <h2 className="text-xl font-bold text-gray-900">Choose Your Theme</h2>
              <p className="mb-4 mt-2 text-sm text-gray-600">
                Select how you want MuttonText to look.
              </p>
              <div className="flex gap-3">
                {(["system", "light", "dark"] as const).map((t) => (
                  <button
                    key={t}
                    onClick={() => setTheme(t)}
                    className={`rounded border px-5 py-3 text-sm capitalize transition-colors ${
                      theme === t
                        ? "border-blue-500 bg-blue-50 text-blue-700"
                        : "border-gray-300 bg-white text-gray-700 hover:bg-gray-50"
                    }`}
                  >
                    {t}
                  </button>
                ))}
              </div>
            </>
          )}

          {step === 3 && (
            <>
              <h2 className="text-xl font-bold text-gray-900">Import Existing Data</h2>
              <p className="mb-4 mt-2 text-sm text-gray-600">
                You can import combos from other text expansion tools, or start fresh.
              </p>
              <div className="flex flex-col gap-2">
                <button className="rounded border border-gray-300 px-4 py-2 text-sm text-gray-700 hover:bg-gray-50">
                  Import from file...
                </button>
                <p className="text-xs text-gray-400">
                  Supports MuttonText JSON, BeefText, and TextExpander formats.
                </p>
              </div>
            </>
          )}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between border-t px-6 py-3">
          <button
            onClick={step === 1 ? onComplete : handleBack}
            className="rounded border border-gray-300 px-4 py-1.5 text-sm text-gray-600 hover:bg-gray-50"
          >
            {step === 1 ? "Skip" : "Back"}
          </button>
          <button
            onClick={handleNext}
            className="rounded bg-blue-600 px-4 py-1.5 text-sm text-white hover:bg-blue-700"
          >
            {step === 3 ? "Get Started" : "Next"}
          </button>
        </div>
      </div>
    </div>
  );
};
