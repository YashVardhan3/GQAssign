# Use an image with Solana and Anchor pre-installed
# This image is commonly used for Anchor development
FROM backpackapp/build:v0.29.0

WORKDIR /workdir

# Copy the program code
COPY . .

# Default command to build
CMD ["anchor", "build"]
