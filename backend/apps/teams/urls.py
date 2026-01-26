"""Team URL patterns."""

from django.urls import path

from .views import (
    TeamDetailView,
    TeamInviteView,
    TeamLeaveView,
    TeamListCreateView,
    TeamRemoveMemberView,
)

urlpatterns = [
    path("", TeamListCreateView.as_view(), name="team-list"),
    path("<int:pk>/", TeamDetailView.as_view(), name="team-detail"),
    path("<int:pk>/invite/", TeamInviteView.as_view(), name="team-invite"),
    path(
        "<int:pk>/members/<int:user_id>/", TeamRemoveMemberView.as_view(), name="team-remove-member"
    ),
    path("<int:pk>/leave/", TeamLeaveView.as_view(), name="team-leave"),
]
