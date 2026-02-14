"""Updates URL patterns."""

from django.urls import path

from .views import LatestUpdateView

urlpatterns = [
    path("latest", LatestUpdateView.as_view(), name="updates-latest"),
]
