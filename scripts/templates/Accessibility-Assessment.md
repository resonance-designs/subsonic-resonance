# WCAG 2.2 AA Accessibility Assessment

**Website:** {{WEBSITE}}  
**Assessment date:** {{ASSESSMENT_DATE}}  
**Target:** {{TARGET}}  
**Overall automated assessment score:** **{{SCORE}}/100**  
**Configured Lighthouse threshold:** **{{THRESHOLD}}/100**  
**Conformance assessment:** **{{CONFORMANCE_STATUS}}**

## Executive summary

{{EXECUTIVE_SUMMARY}}

This document follows the project's full accessibility-assessment format, but its findings are generated from Lighthouse evidence. Automated testing covers only a subset of WCAG success criteria and must be supplemented by expert manual testing.

## Scope and coverage

| Area | URL tested | Test method |
|---|---|---|
| {{TARGET}} | {{WEBSITE}} | Lighthouse automated accessibility audit |

### Automated coverage

- **{{PASSED_AUDITS}}** passed.
- **{{FAILED_AUDITS}}** failed.
- **{{MANUAL_CHECKS}}** require human evaluation.

### Not covered

- Routes and application states not represented by the audited URL.
- Authenticated, provider-specific, error, loading, empty, and modal states not active during the run.
- Complete keyboard-only navigation, focus order, focus visibility, and focus restoration.
- Screen-reader announcements, accessible names in context, and reading order.
- Zoom, reflow, orientation, reduced-motion, high-contrast, and platform accessibility settings.
- End-to-end task completion and WCAG criteria that require human judgment.

## Scoring method

The reported score is Lighthouse's automated accessibility-category score. It is not combined with a manual score and must not be interpreted as a percentage of WCAG 2.2 AA conformance. The configured threshold is a project quality gate, not a legal or standards conformance threshold.

## Findings

{{FINDINGS}}

## Positive observations

{{POSITIVE_OBSERVATIONS}}

## Remediation priority

{{REMEDIATION_PRIORITIES}}

## Recommended conformance statement

As of {{ASSESSMENT_DATE}}, {{TARGET}} received a Lighthouse automated accessibility score of **{{SCORE}}/100** for {{WEBSITE}}. Full WCAG 2.2 Level AA conformance has **not** been established by this automated assessment. Publish a conformance claim only after the failed automated checks are resolved and the excluded manual checks and user workflows have been evaluated.

## Limitations

This assessment is a point-in-time automated review of one rendered URL. Results may change with content, browser, viewport, authentication state, provider data, dependencies, or application updates. Lighthouse cannot validate every WCAG 2.2 success criterion, usability for people with disabilities, or compatibility across assistive technologies. Manual testing and periodic reassessment remain necessary.
