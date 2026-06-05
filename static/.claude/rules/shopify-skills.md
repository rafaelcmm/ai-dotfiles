---
name: shopify-skills
description: Always use Shopify external skills first when the request targets Shopify APIs, extensions, themes, or platform workflows.
applyTo: "**"
---

# Shopify Skills Rule

Sync note: this rule must stay semantically identical across Claude rules and Copilot instructions; update both files in the same change.

When the request includes Shopify context, always consult the Shopify external skills before generic skills.

## Activation

Activate this rule when request mentions Shopify, Hydrogen, Liquid, Polaris, Storefront GraphQL, Functions, Payments Apps, Partner API, Customer API, POS UI, admin extensions, checkout extensions, customer account extensions, metafields, or metaobjects.

## Activation Scope

Activate only when request primarily targets Shopify platform work, not when Shopify is mentioned in passing.

## Mandatory behavior

- Resolve Shopify scope first, then load only the most specific Shopify skill or minimal set of Shopify skills that match the request:
  - `shopify-admin`
  - `shopify-custom-data`
  - `shopify-customer`
  - `shopify-dev`
  - `shopify-functions`
  - `shopify-hydrogen`
  - `shopify-liquid`
  - `shopify-partner`
  - `shopify-payments-apps`
  - `shopify-polaris-admin-extensions`
  - `shopify-polaris-app-home`
  - `shopify-polaris-checkout-extensions`
  - `shopify-polaris-customer-account-extensions`
  - `shopify-pos-ui`
  - `shopify-storefront-graphql`
- Prefer the most specific Shopify skill for the user request.
- Use `shopify-dev` only when no API-specific Shopify skill applies.
- If request spans multiple Shopify domains, combine relevant Shopify skills before falling back to generic cross-domain skills.

## Priority

Shopify-specific skill guidance overrides generic framework guidance only within Shopify domain decisions, and never overrides user intent, safety constraints, or higher-priority workflow contracts.
