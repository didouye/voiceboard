"""Tests for Team models."""

import pytest
from django.contrib.auth import get_user_model
from django.db import IntegrityError

from apps.teams.models import Team, TeamInvitation, TeamMembership

User = get_user_model()


@pytest.mark.django_db
class TestTeamModel:
    """Tests for Team model."""

    def test_create_team(self):
        """Should create a team with owner."""
        owner = User.objects.create_user(email="owner@test.com", password="test")
        team = Team.objects.create(name="Test Team", owner=owner)

        assert team.name == "Test Team"
        assert team.owner == owner
        assert team.max_members == 8
        assert team.member_count == 1  # Owner counts

    def test_max_members_with_extra_seats(self):
        """Should include extra seats in max members."""
        owner = User.objects.create_user(email="owner@test.com", password="test")
        team = Team.objects.create(name="Test Team", owner=owner, extra_seats=5)

        assert team.max_members == 13  # 8 + 5

    def test_member_count_includes_members(self):
        """Should count owner + members."""
        owner = User.objects.create_user(email="owner@test.com", password="test")
        member = User.objects.create_user(email="member@test.com", password="test")
        team = Team.objects.create(name="Test Team", owner=owner)
        TeamMembership.objects.create(team=team, user=member)

        assert team.member_count == 2  # Owner + 1 member


@pytest.mark.django_db
class TestTeamMembership:
    """Tests for TeamMembership model."""

    def test_create_membership(self):
        """Should create membership."""
        owner = User.objects.create_user(email="owner@test.com", password="test")
        member = User.objects.create_user(email="member@test.com", password="test")
        team = Team.objects.create(name="Test Team", owner=owner)

        membership = TeamMembership.objects.create(team=team, user=member)

        assert membership.role == "member"
        assert member in team.members.all()

    def test_unique_membership(self):
        """Should not allow duplicate memberships."""
        owner = User.objects.create_user(email="owner@test.com", password="test")
        member = User.objects.create_user(email="member@test.com", password="test")
        team = Team.objects.create(name="Test Team", owner=owner)
        TeamMembership.objects.create(team=team, user=member)

        with pytest.raises(IntegrityError):
            TeamMembership.objects.create(team=team, user=member)


@pytest.mark.django_db
class TestTeamInvitation:
    """Tests for TeamInvitation model."""

    def test_create_invitation(self):
        """Should create invitation."""
        owner = User.objects.create_user(email="owner@test.com", password="test")
        team = Team.objects.create(name="Test Team", owner=owner)

        invitation = TeamInvitation.objects.create(
            team=team,
            email="invited@test.com",
            invited_by=owner,
            token="test-token-123",
        )

        assert invitation.email == "invited@test.com"
        assert invitation.team == team
