"""Teams admin configuration."""

from django.contrib import admin

from .models import Team, TeamInvitation, TeamMembership


class TeamMembershipInline(admin.TabularInline):
    model = TeamMembership
    extra = 0


@admin.register(Team)
class TeamAdmin(admin.ModelAdmin):
    list_display = ["name", "owner", "member_count", "created_at"]
    search_fields = ["name", "owner__email"]
    inlines = [TeamMembershipInline]


@admin.register(TeamInvitation)
class TeamInvitationAdmin(admin.ModelAdmin):
    list_display = ["email", "team", "invited_by", "created_at"]
    search_fields = ["email", "team__name"]
