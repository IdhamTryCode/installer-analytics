FROM busybox

WORKDIR /app
COPY init.sh ./
# Remove CRLF line endings (works on both busybox and full Linux systems)
# Using tr to remove carriage returns, which is more portable than sed -i
RUN tr -d '\r' < /app/init.sh > /app/init.sh.tmp && mv /app/init.sh.tmp /app/init.sh && chmod +x /app/init.sh