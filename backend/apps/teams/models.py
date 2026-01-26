"""Team models."""

from django.conf import settings
from django.db import models


class Team(models.Model):
    """Team model for collaborative soundboards."""

    name = models.CharField(max_length=100)
    owner = models.ForeignKey(
        settings.AUTH_USER_MODEL,
        on_delete=models.CASCADE,
        related_name="owned_teams",
    )
    members = models.ManyToManyField(
        settings.AUTH_USER_MODEL,
        through="TeamMembership",
        related_name="teams",
    )
    created_at = models.DateTimeField(auto_now_add=True)

    # Stripe
    stripe_subscription_id = models.CharField(max_length=255, blank=True, default="")
    extra_seats = models.PositiveIntegerField(default=0)

    class Meta:
        db_table = "teams"

    def __str__(self):
        return self.name

    @property
    def max_members(self):
        """Maximum members including owner (8 base + extra seats)."""
        return 8 + self.extra_seats

    @property
    def member_count(self):
        """Current member count including owner."""
        return self.members.count() + 1


class TeamMembership(models.Model):
    """Through model for team membership."""

    ROLE_MEMBER = "member"
    ROLE_CHOICES = [
        (ROLE_MEMBER, "Member"),
    ]

    team = models.ForeignKey(Team, on_delete=models.CASCADE)
    user = models.ForeignKey(settings.AUTH_USER_MODEL, on_delete=models.CASCADE)
    role = models.CharField(max_length=20, choices=ROLE_CHOICES, default=ROLE_MEMBER)
    joined_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        db_table = "team_memberships"
        unique_together = ["team", "user"]


class TeamInvitation(models.Model):
    """Pending team invitation."""

    team = models.ForeignKey(Team, on_delete=models.CASCADE, related_name="invitations")
    email = models.EmailField()
    invited_by = models.ForeignKey(
        settings.AUTH_USER_MODEL,
        on_delete=models.CASCADE,
        related_name="sent_invitations",
    )
    token = models.CharField(max_length=64, unique=True)
    created_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        db_table = "team_invitations"
        unique_together = ["team", "email"]
