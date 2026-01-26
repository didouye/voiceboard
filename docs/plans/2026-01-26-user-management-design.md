# User Management Design

> **Date:** 2026-01-26
> **Status:** Ready for implementation
> **Phase:** 4 - Cloud & Collaboration

## Overview

This document describes the user management system for Voiceboard, including authentication, subscriptions, billing, and team management.

## Decisions Summary

| Aspect | Decision |
|--------|----------|
| Licensing model | Freemium + Teams |
| Premium pricing | 0.99€/month, 9.49€/year |
| Teams pricing | 3.99€/month (≤8 users), +0.49€/extra user |
| Super admin | Environment variable `SUPER_ADMIN_EMAIL` |
| Account creation | OAuth only (Google, Discord) |
| Profile fields | Standard (display_name, timezone, language) |
| Payments | Stripe Checkout + Customer Portal |
| Expiration | 7-day grace period, read-only data after downgrade |
| Team billing | Owner-only |
| Team roles | Owner + Member |

## Feature Distribution

| Feature | Free | Premium | Teams |
|---------|------|---------|-------|
| Unlimited pads | ✅ | ✅ | ✅ |
| Unlimited folders | ✅ | ✅ | ✅ |
| Keyboard shortcuts | ✅ | ✅ | ✅ |
| Pad images | ✅ | ✅ | ✅ |
| Cloud sync | ❌ | ✅ | ✅ |
| Sound search APIs | ❌ | ✅ | ✅ |
| AI generation | ❌ | ✅ | ✅ |
| Remote control (mobile) | ❌ | ✅ | ✅ |
| Shared soundboards | ❌ | ❌ | ✅ |
| Team members | ❌ | ❌ | ✅ |
| Discord bot | ❌ | ❌ | ✅ |

## Pricing Structure

### Premium (Personal)

| Billing | Price | Per month |
|---------|-------|-----------|
| Monthly | 0.99€ | 0.99€ |
| Yearly | 9.49€ | 0.79€ (-20%) |

### Teams

| Billing | Base (≤8 users) | Extra seat |
|---------|-----------------|------------|
| Monthly | 3.99€ | +0.49€ |
| Yearly | 38.30€ | +4.70€ |

## Data Models

### User Model (Extended)

```python
# apps/users/models.py
class User(AbstractUser):
    # Existing fields
    username = None
    email = models.EmailField("email address", unique=True)
    avatar_url = models.URLField(blank=True, default="")
    google_id = models.CharField(max_length=255, blank=True, default="")
    discord_id = models.CharField(max_length=255, blank=True, default="")

    # New profile fields
    display_name = models.CharField(max_length=50, blank=True, default="")
    timezone = models.CharField(max_length=50, default="UTC")
    language = models.CharField(max_length=10, default="en")

    # Subscription fields
    stripe_customer_id = models.CharField(max_length=255, blank=True, default="")
    subscription_tier = models.CharField(
        max_length=20,
        choices=[("free", "Free"), ("premium", "Premium"), ("teams", "Teams")],
        default="free"
    )
    subscription_status = models.CharField(
        max_length=20,
        choices=[
            ("none", "None"),
            ("active", "Active"),
            ("past_due", "Past Due"),
            ("cancelled", "Cancelled"),
        ],
        default="none"
    )
    subscription_ends_at = models.DateTimeField(null=True, blank=True)
```

### Team Model

```python
# apps/teams/models.py
class Team(models.Model):
    name = models.CharField(max_length=100)
    owner = models.ForeignKey(
        User, on_delete=models.CASCADE, related_name="owned_teams"
    )
    members = models.ManyToManyField(
        User, through="TeamMembership", related_name="teams"
    )
    created_at = models.DateTimeField(auto_now_add=True)

    # Stripe
    stripe_subscription_id = models.CharField(max_length=255, blank=True, default="")
    extra_seats = models.PositiveIntegerField(default=0)

    class Meta:
        db_table = "teams"

    @property
    def max_members(self):
        """Maximum members including owner (8 base + extra seats)."""
        return 8 + self.extra_seats

    @property
    def member_count(self):
        """Current member count including owner."""
        return self.members.count() + 1


class TeamMembership(models.Model):
    team = models.ForeignKey(Team, on_delete=models.CASCADE)
    user = models.ForeignKey(User, on_delete=models.CASCADE)
    role = models.CharField(
        max_length=20,
        choices=[("member", "Member")],
        default="member"
    )
    joined_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        db_table = "team_memberships"
        unique_together = ["team", "user"]


class TeamInvitation(models.Model):
    team = models.ForeignKey(Team, on_delete=models.CASCADE)
    email = models.EmailField()
    invited_by = models.ForeignKey(User, on_delete=models.CASCADE)
    token = models.CharField(max_length=64, unique=True)
    created_at = models.DateTimeField(auto_now_add=True)
    expires_at = models.DateTimeField()  # +7 days from created_at

    class Meta:
        db_table = "team_invitations"
        unique_together = ["team", "email"]
```

## API Endpoints

### Authentication (Existing)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/auth/me/` | Get current user profile |
| PUT | `/api/auth/me/` | Update profile |
| POST | `/api/auth/logout/` | Logout (blacklist token) |
| POST | `/api/auth/refresh/` | Refresh JWT token |
| GET | `/api/auth/google/url/` | Get Google OAuth URL |
| POST | `/api/auth/google/callback/` | Google OAuth callback |
| GET | `/api/auth/discord/url/` | Get Discord OAuth URL |
| POST | `/api/auth/discord/callback/` | Discord OAuth callback |

### Billing (New)

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/billing/checkout/` | Create Stripe Checkout session |
| POST | `/api/billing/portal/` | Get Stripe Customer Portal URL |
| GET | `/api/billing/subscription/` | Get current subscription status |
| POST | `/api/billing/webhook/` | Stripe webhook (no JWT auth) |

**Checkout Request:**
```json
{
  "plan": "premium_monthly" | "premium_yearly" | "teams_monthly" | "teams_yearly"
}
```

**Checkout Response:**
```json
{
  "checkout_url": "https://checkout.stripe.com/c/pay/cs_xxx"
}
```

**Subscription Response:**
```json
{
  "tier": "premium",
  "status": "active",
  "ends_at": "2026-02-26T00:00:00Z",
  "plan": "monthly"
}
```

### Teams (New)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/teams/` | List my teams (owned + member) |
| POST | `/api/teams/` | Create team (requires Teams subscription) |
| GET | `/api/teams/{id}/` | Get team details |
| PUT | `/api/teams/{id}/` | Update team (owner only) |
| DELETE | `/api/teams/{id}/` | Delete team (owner only) |
| POST | `/api/teams/{id}/invite/` | Invite member by email (owner only) |
| DELETE | `/api/teams/{id}/members/{uid}/` | Remove member (owner only) |
| POST | `/api/teams/{id}/leave/` | Leave team (member only) |

## Super Admin Setup

Super admin is created automatically on first deployment via environment variable.

```python
# apps/core/management/commands/setup_superadmin.py
from django.core.management.base import BaseCommand
from django.contrib.auth import get_user_model
import os

User = get_user_model()

class Command(BaseCommand):
    help = "Create super admin from SUPER_ADMIN_EMAIL env var"

    def handle(self, *args, **options):
        email = os.environ.get("SUPER_ADMIN_EMAIL")
        if not email:
            self.stdout.write("SUPER_ADMIN_EMAIL not set, skipping")
            return

        if User.objects.filter(is_superuser=True).exists():
            self.stdout.write("Super admin already exists, skipping")
            return

        user, created = User.objects.get_or_create(
            email=email,
            defaults={
                "is_staff": True,
                "is_superuser": True,
                "subscription_tier": "teams",  # Full access
                "subscription_status": "active",
            }
        )

        if created:
            self.stdout.write(self.style.SUCCESS(f"Super admin created: {email}"))
        else:
            user.is_staff = True
            user.is_superuser = True
            user.save()
            self.stdout.write(self.style.SUCCESS(f"Super admin promoted: {email}"))
```

Called in `docker-entrypoint.sh`:
```bash
python manage.py migrate
python manage.py setup_superadmin
```

## Stripe Webhook Handling

| Event | Action |
|-------|--------|
| `checkout.session.completed` | Create/update subscription, set tier |
| `customer.subscription.updated` | Update status, ends_at |
| `customer.subscription.deleted` | Set status=cancelled, schedule downgrade |
| `invoice.payment_failed` | Set status=past_due |
| `invoice.paid` | Set status=active |

```python
# apps/billing/views.py
import stripe
from django.conf import settings
from django.http import HttpResponse
from django.views.decorators.csrf import csrf_exempt
from django.views.decorators.http import require_POST

@csrf_exempt
@require_POST
def stripe_webhook(request):
    payload = request.body
    sig_header = request.headers.get("Stripe-Signature")

    try:
        event = stripe.Webhook.construct_event(
            payload, sig_header, settings.STRIPE_WEBHOOK_SECRET
        )
    except (ValueError, stripe.error.SignatureVerificationError):
        return HttpResponse(status=400)

    handler = WEBHOOK_HANDLERS.get(event["type"])
    if handler:
        handler(event["data"]["object"])

    return HttpResponse(status=200)
```

## Grace Period and Read-Only Mode

### Expiration Processing

Daily task to downgrade users after 7-day grace period:

```python
# apps/billing/tasks.py
from datetime import timedelta
from django.utils import timezone
from django.contrib.auth import get_user_model

User = get_user_model()

def process_expired_subscriptions():
    """Run daily via cron/celery beat."""
    grace_period = timedelta(days=7)
    cutoff = timezone.now() - grace_period

    expired_users = User.objects.filter(
        subscription_status="cancelled",
        subscription_ends_at__lt=cutoff,
        subscription_tier__in=["premium", "teams"]
    )

    for user in expired_users:
        user.subscription_tier = "free"
        user.subscription_status = "none"
        user.save()
        # TODO: Send downgrade notification email
```

### Read-Only Permission

```python
# apps/core/permissions.py
from rest_framework import permissions

class CanModifyCloudData(permissions.BasePermission):
    """Allow modification only with active subscription."""

    message = "Your subscription has expired. Your data is read-only."

    def has_permission(self, request, view):
        if request.method in permissions.SAFE_METHODS:
            return True  # GET, HEAD, OPTIONS always allowed

        user = request.user
        if user.subscription_tier == "free":
            return False

        if user.subscription_status in ["cancelled", "past_due"]:
            return False

        return True
```

## Team Invitation Flow

```
1. Owner calls POST /api/teams/{id}/invite/ with { "email": "user@example.com" }

2. Backend validates:
   - Owner has active Teams subscription
   - Team has not reached max_members
   - User is not already a member
   - No pending invitation for this email

3. If user exists in database:
   - Add directly to team as member
   - Send "You've been added" email

4. If user does not exist:
   - Create TeamInvitation with token
   - Send invitation email with link

5. When invited user signs up (OAuth):
   - Check for pending invitations by email
   - Auto-accept all pending invitations
   - Delete invitation records
```

## Feature Permission Helper

```python
# apps/core/permissions.py
FEATURE_TIERS = {
    "cloud_sync": ["premium", "teams"],
    "sound_search": ["premium", "teams"],
    "ai_generation": ["premium", "teams"],
    "remote_control": ["premium", "teams"],
    "shared_soundboards": ["teams"],
    "team_management": ["teams"],
    "discord_bot": ["teams"],
}

def user_has_feature(user, feature):
    """Check if user has access to a feature."""
    required_tiers = FEATURE_TIERS.get(feature, [])
    if not required_tiers:
        return True  # Feature not restricted
    return user.subscription_tier in required_tiers

def can_manage_team(user, team):
    """Check if user is team owner."""
    return team.owner_id == user.id
```

## File Structure

```
backend/
├── apps/
│   ├── users/              # Existing - modify
│   │   ├── models.py       # Add profile + subscription fields
│   │   ├── serializers.py  # Update UserSerializer
│   │   └── migrations/
│   ├── billing/            # New app
│   │   ├── __init__.py
│   │   ├── admin.py
│   │   ├── constants.py    # Price IDs, tiers
│   │   ├── models.py       # Empty for now
│   │   ├── serializers.py
│   │   ├── services.py     # Stripe logic
│   │   ├── tasks.py        # Expiration processing
│   │   ├── urls.py
│   │   └── views.py
│   ├── teams/              # New app
│   │   ├── __init__.py
│   │   ├── admin.py
│   │   ├── models.py       # Team, TeamMembership, TeamInvitation
│   │   ├── permissions.py
│   │   ├── serializers.py
│   │   ├── urls.py
│   │   └── views.py
│   └── core/
│       ├── permissions.py  # Feature permissions
│       └── management/
│           └── commands/
│               └── setup_superadmin.py
├── config/
│   └── settings/
│       └── base.py         # Add STRIPE_* settings
```

## Dependencies

```toml
# pyproject.toml
[project.dependencies]
stripe = ">=7.0.0"
```

## Environment Variables

```bash
# Super admin
SUPER_ADMIN_EMAIL=admin@voiceboard.app

# Stripe
STRIPE_SECRET_KEY=sk_live_xxx
STRIPE_PUBLISHABLE_KEY=pk_live_xxx
STRIPE_WEBHOOK_SECRET=whsec_xxx

# Stripe Price IDs (create in Stripe Dashboard)
STRIPE_PRICE_PREMIUM_MONTHLY=price_xxx
STRIPE_PRICE_PREMIUM_YEARLY=price_xxx
STRIPE_PRICE_TEAMS_MONTHLY=price_xxx
STRIPE_PRICE_TEAMS_YEARLY=price_xxx
STRIPE_PRICE_EXTRA_SEAT_MONTHLY=price_xxx
STRIPE_PRICE_EXTRA_SEAT_YEARLY=price_xxx
```

## Implementation Order

1. **User model updates** - Add new fields, migration
2. **Super admin command** - Management command + entrypoint
3. **Billing app** - Stripe integration, checkout, portal, webhooks
4. **Teams app** - Models, CRUD, invitations
5. **Permissions** - Feature gates, read-only mode
6. **Tasks** - Expiration processing (cron or celery)
