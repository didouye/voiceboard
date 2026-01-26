"""Custom User model - email-based authentication."""

from django.contrib.auth.models import AbstractUser, BaseUserManager
from django.db import models


class UserManager(BaseUserManager):
    """Custom user manager for email-based authentication."""

    def create_user(self, email, password=None, **extra_fields):
        """Create and return a regular user."""
        if not email:
            raise ValueError("Email is required")
        email = self.normalize_email(email)
        user = self.model(email=email, **extra_fields)
        user.set_password(password)
        user.save(using=self._db)
        return user

    def create_superuser(self, email, password=None, **extra_fields):
        """Create and return a superuser."""
        extra_fields.setdefault("is_staff", True)
        extra_fields.setdefault("is_superuser", True)

        if extra_fields.get("is_staff") is not True:
            raise ValueError("Superuser must have is_staff=True")
        if extra_fields.get("is_superuser") is not True:
            raise ValueError("Superuser must have is_superuser=True")

        return self.create_user(email, password, **extra_fields)


class User(AbstractUser):
    """Custom user model - email-based, no username."""

    # Tier choices
    TIER_FREE = "free"
    TIER_PREMIUM = "premium"
    TIER_TEAMS = "teams"
    TIER_CHOICES = [
        (TIER_FREE, "Free"),
        (TIER_PREMIUM, "Premium"),
        (TIER_TEAMS, "Teams"),
    ]

    # Subscription status choices
    STATUS_NONE = "none"
    STATUS_ACTIVE = "active"
    STATUS_PAST_DUE = "past_due"
    STATUS_CANCELLED = "cancelled"
    STATUS_CHOICES = [
        (STATUS_NONE, "None"),
        (STATUS_ACTIVE, "Active"),
        (STATUS_PAST_DUE, "Past Due"),
        (STATUS_CANCELLED, "Cancelled"),
    ]

    # Auth fields
    username = None
    email = models.EmailField("email address", unique=True)

    # OAuth provider IDs
    google_id = models.CharField(max_length=255, blank=True, default="")
    discord_id = models.CharField(max_length=255, blank=True, default="")

    # Profile fields
    avatar_url = models.URLField(blank=True, default="")
    display_name = models.CharField(max_length=50, blank=True, default="")
    timezone = models.CharField(max_length=50, default="UTC")
    language = models.CharField(max_length=10, default="en")

    # Subscription fields
    stripe_customer_id = models.CharField(max_length=255, blank=True, default="")
    subscription_tier = models.CharField(max_length=20, choices=TIER_CHOICES, default=TIER_FREE)
    subscription_status = models.CharField(
        max_length=20, choices=STATUS_CHOICES, default=STATUS_NONE
    )
    subscription_ends_at = models.DateTimeField(null=True, blank=True)

    objects = UserManager()

    USERNAME_FIELD = "email"
    REQUIRED_FIELDS = []

    class Meta:
        db_table = "users"

    def __str__(self):
        return self.email

    @property
    def has_active_subscription(self):
        """Check if user has an active paid subscription."""
        return (
            self.subscription_tier in [self.TIER_PREMIUM, self.TIER_TEAMS]
            and self.subscription_status == self.STATUS_ACTIVE
        )
