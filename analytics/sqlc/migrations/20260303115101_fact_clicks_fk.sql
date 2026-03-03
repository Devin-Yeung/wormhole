-- +goose Up
-- +goose StatementBegin
ALTER TABLE fact_clicks
    ADD CONSTRAINT fk_fact_clicks_visitor FOREIGN KEY (visitor_key) REFERENCES dim_visitors (visitor_key);
-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
ALTER TABLE fact_clicks
    DROP FOREIGN KEY fk_fact_clicks_visitor;
-- +goose StatementEnd
