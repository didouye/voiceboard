"""Management command to create super admin from environment variable."""

from django.conf import settings
from django.contrib.auth import get_user_model
from django.core.management.base import BaseCommand

User = get_user_model()


class Command(BaseCommand):
    """Create super admin from SUPER_ADMIN_EMAIL environment variable."""

    help = "Create super admin from SUPER_ADMIN_EMAIL env var"

    def handle(self, *args, **options):
        """Execute the command."""
        email = settings.SUPER_ADMIN_EMAIL

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
                "subscription_tier": User.TIER_TEAMS,
                "subscription_status": User.STATUS_ACTIVE,
            },
        )

        if created:
            self.stdout.write(self.style.SUCCESS(f"Super admin created: {email}"))
        else:
            user.is_staff = True
            user.is_superuser = True
            user.subscription_tier = User.TIER_TEAMS
            user.subscription_status = User.STATUS_ACTIVE
            user.save()
            self.stdout.write(self.style.SUCCESS(f"Super admin promoted: {email}"))
