FROM python:3.12-alpine

RUN pip install flask

WORKDIR /app

COPY app.py /app/app.py

ENV FLASK_APP=app.py

CMD ["python", "-m", "flask", "run", "--host=0.0.0.0", "--port=8080"]
