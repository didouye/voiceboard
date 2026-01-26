"""Team views."""

import secrets

from django.contrib.auth import get_user_model
from rest_framework import generics, status
from rest_framework.permissions import IsAuthenticated
from rest_framework.response import Response
from rest_framework.views import APIView

from .models import Team, TeamInvitation, TeamMembership
from .permissions import HasTeamsSubscription, IsTeamMember, IsTeamOwner
from .serializers import (
    InviteSerializer,
    TeamCreateSerializer,
    TeamSerializer,
)

User = get_user_model()


class TeamListCreateView(generics.ListCreateAPIView):
    """List user's teams or create a new team."""

    permission_classes = [IsAuthenticated]

    def get_serializer_class(self):
        if self.request.method == "POST":
            return TeamCreateSerializer
        return TeamSerializer

    def get_queryset(self):
        user = self.request.user
        # Teams where user is owner or member
        owned = Team.objects.filter(owner=user)
        member_of = Team.objects.filter(members=user)
        return (owned | member_of).distinct()

    def get_permissions(self):
        if self.request.method == "POST":
            return [IsAuthenticated(), HasTeamsSubscription()]
        return [IsAuthenticated()]

    def perform_create(self, serializer):
        serializer.save(owner=self.request.user)


class TeamDetailView(generics.RetrieveUpdateDestroyAPIView):
    """Get, update, or delete a team."""

    serializer_class = TeamSerializer
    permission_classes = [IsAuthenticated, IsTeamMember]

    def get_queryset(self):
        user = self.request.user
        owned = Team.objects.filter(owner=user)
        member_of = Team.objects.filter(members=user)
        return (owned | member_of).distinct()

    def get_permissions(self):
        if self.request.method in ["PUT", "PATCH", "DELETE"]:
            return [IsAuthenticated(), IsTeamOwner()]
        return [IsAuthenticated(), IsTeamMember()]


class TeamInviteView(APIView):
    """Invite a member to a team."""

    permission_classes = [IsAuthenticated, IsTeamOwner]

    def post(self, request, pk):
        try:
            team = Team.objects.get(pk=pk)
        except Team.DoesNotExist:
            return Response(
                {"error": "Team not found"},
                status=status.HTTP_404_NOT_FOUND,
            )

        self.check_object_permissions(request, team)

        serializer = InviteSerializer(data=request.data)
        if not serializer.is_valid():
            return Response(serializer.errors, status=status.HTTP_400_BAD_REQUEST)

        email = serializer.validated_data["email"]

        # Check team capacity
        if team.member_count >= team.max_members:
            return Response(
                {"error": "Team is at maximum capacity"},
                status=status.HTTP_400_BAD_REQUEST,
            )

        # Check if user exists
        try:
            user = User.objects.get(email=email)
            # Check if already a member
            if user == team.owner or user in team.members.all():
                return Response(
                    {"error": "User is already a team member"},
                    status=status.HTTP_400_BAD_REQUEST,
                )
            # Add directly
            TeamMembership.objects.create(team=team, user=user)
            return Response(
                {"message": f"{email} added to team"},
                status=status.HTTP_201_CREATED,
            )
        except User.DoesNotExist:
            # Create invitation
            if team.invitations.filter(email=email).exists():
                return Response(
                    {"error": "Invitation already sent"},
                    status=status.HTTP_400_BAD_REQUEST,
                )

            TeamInvitation.objects.create(
                team=team,
                email=email,
                invited_by=request.user,
                token=secrets.token_urlsafe(32),
            )
            # TODO: Send invitation email
            return Response(
                {"message": f"Invitation sent to {email}"},
                status=status.HTTP_201_CREATED,
            )


class TeamRemoveMemberView(APIView):
    """Remove a member from a team."""

    permission_classes = [IsAuthenticated, IsTeamOwner]

    def delete(self, request, pk, user_id):
        try:
            team = Team.objects.get(pk=pk)
        except Team.DoesNotExist:
            return Response(
                {"error": "Team not found"},
                status=status.HTTP_404_NOT_FOUND,
            )

        self.check_object_permissions(request, team)

        try:
            membership = TeamMembership.objects.get(team=team, user_id=user_id)
            membership.delete()
            return Response(status=status.HTTP_204_NO_CONTENT)
        except TeamMembership.DoesNotExist:
            return Response(
                {"error": "User is not a team member"},
                status=status.HTTP_404_NOT_FOUND,
            )


class TeamLeaveView(APIView):
    """Leave a team."""

    permission_classes = [IsAuthenticated]

    def post(self, request, pk):
        try:
            team = Team.objects.get(pk=pk)
        except Team.DoesNotExist:
            return Response(
                {"error": "Team not found"},
                status=status.HTTP_404_NOT_FOUND,
            )

        if team.owner == request.user:
            return Response(
                {"error": "Owner cannot leave the team. Delete it instead."},
                status=status.HTTP_400_BAD_REQUEST,
            )

        try:
            membership = TeamMembership.objects.get(team=team, user=request.user)
            membership.delete()
            return Response(status=status.HTTP_204_NO_CONTENT)
        except TeamMembership.DoesNotExist:
            return Response(
                {"error": "You are not a member of this team"},
                status=status.HTTP_400_BAD_REQUEST,
            )
