"""Billing constants."""

from django.conf import settings

# Valid plan identifiers
VALID_PLANS = [
    "premium_monthly",
    "premium_yearly",
    "teams_monthly",
    "teams_yearly",
]


def get_price_id(plan: str) -> str:
    """Get Stripe price ID for a plan."""
    return settings.STRIPE_PRICES.get(plan, "")


def get_tier_for_plan(plan: str) -> str:
    """Get subscription tier for a plan."""
    if plan.startswith("premium"):
        return "premium"
    elif plan.startswith("teams"):
        return "teams"
    return "free"
