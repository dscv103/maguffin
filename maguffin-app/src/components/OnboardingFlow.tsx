import { useState, useEffect } from "react";

interface OnboardingStep {
  id: string;
  title: string;
  description: string;
  icon: string;
  tips?: string[];
}

const ONBOARDING_STEPS: OnboardingStep[] = [
  {
    id: "welcome",
    title: "Welcome to Maguffin! 👋",
    description: "A modern Git client with Tower-style PR dashboard and Graphite-style stacked PR workflow.",
    icon: "🎉",
    tips: [
      "Fast and native desktop experience",
      "Integrated GitHub pull request management",
      "Powerful stacked PR workflow for complex features"
    ]
  },
  {
    id: "auth",
    title: "Connect to GitHub",
    description: "Sign in with your GitHub account to access your repositories and pull requests.",
    icon: "🔐",
    tips: [
      "We use GitHub's secure device flow for authentication",
      "Your token is stored safely in your system's keychain",
      "You can sign out anytime from the settings"
    ]
  },
  {
    id: "repo",
    title: "Open a Repository",
    description: "Select a local Git repository with a GitHub remote to get started.",
    icon: "📁",
    tips: [
      "Enter the full path to your local git repository",
      "The repository must have a GitHub remote configured",
      "Your recent repositories will be saved for quick access"
    ]
  },
  {
    id: "prs",
    title: "Manage Pull Requests",
    description: "View, checkout, merge, and close pull requests directly from the app.",
    icon: "📋",
    tips: [
      "Click on a PR to view details and take actions",
      "Use keyboard shortcuts for faster navigation (press ? for help)",
      "Sort and filter PRs by status, date, or activity"
    ]
  },
  {
    id: "stacks",
    title: "Stacked PR Workflow",
    description: "Organize related branches into stacks for complex features with multiple PRs.",
    icon: "📚",
    tips: [
      "Create stacks to track branch dependencies",
      "Automatically restack when parent branches are merged",
      "Keep PRs small and focused with dependent changes"
    ]
  }
];

interface OnboardingFlowProps {
  onComplete: () => void;
  onSkip: () => void;
}

export function OnboardingFlow({ onComplete, onSkip }: OnboardingFlowProps) {
  const [currentStep, setCurrentStep] = useState(0);
  const step = ONBOARDING_STEPS[currentStep];
  const isFirstStep = currentStep === 0;
  const isLastStep = currentStep === ONBOARDING_STEPS.length - 1;

  const handleNext = () => {
    if (isLastStep) {
      localStorage.setItem("maguffin_onboarding_complete", "true");
      onComplete();
    } else {
      setCurrentStep(currentStep + 1);
    }
  };

  const handlePrevious = () => {
    if (!isFirstStep) {
      setCurrentStep(currentStep - 1);
    }
  };

  const handleSkip = () => {
    localStorage.setItem("maguffin_onboarding_complete", "true");
    onSkip();
  };

  // Handle keyboard navigation
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "ArrowRight" || e.key === "Enter") {
        handleNext();
      } else if (e.key === "ArrowLeft" && !isFirstStep) {
        handlePrevious();
      } else if (e.key === "Escape") {
        handleSkip();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [currentStep, isFirstStep, isLastStep]);

  return (
    <div className="onboarding-overlay">
      <div className="onboarding-dialog">
        <header className="onboarding-header">
          <div className="step-indicator">
            {ONBOARDING_STEPS.map((_, index) => (
              <button
                key={index}
                className={`step-dot ${index === currentStep ? "active" : ""} ${index < currentStep ? "completed" : ""}`}
                onClick={() => setCurrentStep(index)}
                aria-label={`Go to step ${index + 1}`}
              />
            ))}
          </div>
          <button className="skip-btn" onClick={handleSkip}>
            Skip tour
          </button>
        </header>

        <div className="onboarding-content">
          <div className="step-icon">{step.icon}</div>
          <h2 className="step-title">{step.title}</h2>
          <p className="step-description">{step.description}</p>

          {step.tips && step.tips.length > 0 && (
            <ul className="step-tips">
              {step.tips.map((tip, index) => (
                <li key={index} className="tip-item">
                  <span className="tip-bullet">•</span>
                  {tip}
                </li>
              ))}
            </ul>
          )}
        </div>

        <footer className="onboarding-footer">
          <button
            className="nav-btn prev-btn"
            onClick={handlePrevious}
            disabled={isFirstStep}
          >
            ← Previous
          </button>
          <span className="step-counter">
            {currentStep + 1} of {ONBOARDING_STEPS.length}
          </span>
          <button className="nav-btn next-btn" onClick={handleNext}>
            {isLastStep ? "Get Started →" : "Next →"}
          </button>
        </footer>
      </div>
    </div>
  );
}

/**
 * Hook to check if onboarding should be shown
 */
export function useOnboarding() {
  const [showOnboarding, setShowOnboarding] = useState(false);

  useEffect(() => {
    const completed = localStorage.getItem("maguffin_onboarding_complete");
    if (!completed) {
      setShowOnboarding(true);
    }
  }, []);

  const completeOnboarding = () => {
    localStorage.setItem("maguffin_onboarding_complete", "true");
    setShowOnboarding(false);
  };

  const resetOnboarding = () => {
    localStorage.removeItem("maguffin_onboarding_complete");
    setShowOnboarding(true);
  };

  return {
    showOnboarding,
    completeOnboarding,
    resetOnboarding
  };
}
