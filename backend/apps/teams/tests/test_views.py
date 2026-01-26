"""Tests for teams views."""

import pytest
from django.contrib.auth import get_user_model
from django.urls import reverse
from rest_framework.test import APIClient

from apps.teams.models import Team, TeamMembership

User = get_user_model()


@pytest.fixture
def api_client():
    return APIClient()


@pytest.fixture
def premium_user(api_client):
    user = User.objects.create_user(
        email="premium@test.com",
        password="test",
        subscription_tier="teams",
        subscription_status="active",
    )
    api_client.force_authenticate(user=user)
    return user


@pytest.fixture
def free_user(api_client):
    user = User.objects.create_user(email="free@test.com", password="test")
    api_client.force_authenticate(user=user)
    return user


@pytest.mark.django_db
class TestTeamListCreate:
    """Tests for team list/create endpoint."""

    def test_list_teams(self, api_client, premium_user):
        """Should list user's teams."""
        Team.objects.create(name="My Team", owner=premium_user)
        response = api_client.get(reverse("team-list"))

        assert response.status_code == 200
        assert len(response.data) == 1
        assert response.data[0]["name"] == "My Team"

    def test_create_team_requires_subscription(self, api_client, free_user):
        """Should require Teams subscription to create team."""
        response = api_client.post(
            reverse("team-list"),
            {"name": "New Team"},
            format="json",
        )
        assert response.status_code == 403

    def test_create_team_with_subscription(self, api_client, premium_user):
        """Should allow creating team with subscription."""
        response = api_client.post(
            reverse("team-list"),
            {"name": "New Team"},
            format="json",
        )
        assert response.status_code == 201
        assert Team.objects.filter(name="New Team", owner=premium_user).exists()


@pytest.mark.django_db
class TestTeamDetail:
    """Tests for team detail endpoint."""

    def test_get_team_detail(self, api_client, premium_user):
        """Should get team details."""
        team = Team.objects.create(name="My Team", owner=premium_user)
        response = api_client.get(reverse("team-detail", args=[team.id]))

        assert response.status_code == 200
        assert response.data["name"] == "My Team"

    def test_update_team_owner_only(self, api_client, premium_user):
        """Only owner can update team."""
        other_user = User.objects.create_user(email="other@test.com", password="test")
        team = Team.objects.create(name="Other Team", owner=other_user)
        TeamMembership.objects.create(team=team, user=premium_user)

        response = api_client.put(
            reverse("team-detail", args=[team.id]),
            {"name": "Renamed"},
            format="json",
        )
        assert response.status_code == 403

    def test_delete_team_owner_only(self, api_client, premium_user):
        """Only owner can delete team."""
        team = Team.objects.create(name="My Team", owner=premium_user)
        response = api_client.delete(reverse("team-detail", args=[team.id]))

        assert response.status_code == 204
        assert not Team.objects.filter(id=team.id).exists()


@pytest.mark.django_db
class TestTeamInvite:
    """Tests for team invite endpoint."""

    def test_invite_member(self, api_client, premium_user):
        """Should invite member by email."""
        team = Team.objects.create(name="My Team", owner=premium_user)
        response = api_client.post(
            reverse("team-invite", args=[team.id]),
            {"email": "newmember@test.com"},
            format="json",
        )

        assert response.status_code == 201
        assert team.invitations.filter(email="newmember@test.com").exists()

    def test_invite_existing_user_adds_directly(self, api_client, premium_user):
        """Should add existing user directly."""
        existing = User.objects.create_user(email="existing@test.com", password="test")
        team = Team.objects.create(name="My Team", owner=premium_user)

        response = api_client.post(
            reverse("team-invite", args=[team.id]),
            {"email": "existing@test.com"},
            format="json",
        )

        assert response.status_code == 201
        assert existing in team.members.all()


@pytest.mark.django_db
class TestTeamLeave:
    """Tests for team leave endpoint."""

    def test_member_can_leave(self, api_client, premium_user):
        """Member should be able to leave team."""
        owner = User.objects.create_user(
            email="owner@test.com",
            password="test",
            subscription_tier="teams",
            subscription_status="active",
        )
        team = Team.objects.create(name="Owner Team", owner=owner)
        TeamMembership.objects.create(team=team, user=premium_user)

        response = api_client.post(reverse("team-leave", args=[team.id]))

        assert response.status_code == 204
        assert premium_user not in team.members.all()

    def test_owner_cannot_leave(self, api_client, premium_user):
        """Owner cannot leave their own team."""
        team = Team.objects.create(name="My Team", owner=premium_user)

        response = api_client.post(reverse("team-leave", args=[team.id]))

        assert response.status_code == 400
