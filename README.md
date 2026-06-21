# Odoo POS Loyalty System with Temporal Saga

A distributed system integrating Odoo POS with a loyalty platform using Kafka, Debezium, and Temporal orchestration.

## To Do

- [x] Odoo + PostgreSQL in Docker Compose
- [x] Enable POS module in Odoo
- [x] Loyalty Backend
- [x] Loyalty Frontend (PWA) (Authenticate with Auth0)
- [x] Capture the session at the POS and link the customer
- [x] Setup Kafka + Debezium PostgreSQL CDC
- [x] Event Processing (Easy-to-Moderate)
- [ ] Build KFunc/transform service (Map events to points earned or redeemed; aggregate them to calculate balance; store balance)
- [ ] Use Temporal.io to orchestrate sign-up between Auth0, Odoo (Optional)




https://developer.confluent.io/courses/event-sourcing/hands-on-confluent-cloud/

```text
2026-06-28T19:17:23.311178Z  INFO consumer: key: 'Some([123, 34, 105, 100, 34, 58, 53, 125])', payload: '{"before":null,"after":{"id":5,"user_id":2,"company_id":1,"pricelist_id":null,"partner_id":12,"sequence_number":8,"session_id":4,"config_id":1,"account_move":null,"procurement_group_id":null,"nb_print":0,"sale_journal":9,"fiscal_position_id":null,"create_uid":2,"write_uid":2,"access_token":"9bafabe9-73c0-4942-9c0b-b77441550dc8","name":"Loyalty/0004","last_order_preparation_change":"{\"lines\":{},\"generalNote\":\"\",\"sittingMode\":\"dine in\"}","state":"paid","floating_order_name":null,"pos_reference":"Order 00004-022-0008","ticket_code":"jgxy7","uuid":"7bd0033a-741d-41c7-a29c-ef0f5a7a93c3","email":"Waqqas Sheikh","mobile":null,"shipping_date":null,"general_note":"","amount_difference":{"scale":0,"value":"AA=="},"amount_tax":{"scale":1,"value":"LQ=="},"amount_total":{"scale":1,"value":"AVk="},"amount_paid":{"scale":1,"value":"AVk="},"amount_return":{"scale":0,"value":"AA=="},"currency_rate":{"scale":0,"value":"AQ=="},"tip_amount":{"scale":0,"value":"AA=="},"to_invoice":false,"is_tipped":false,"has_deleted_line":false,"date_order":1782674242000000,"create_date":1782674242125578,"write_date":1782674242125578,"next_online_payment_amount":{"scale":0,"value":"AA=="},"loyalty_session_code":"LOY-DYRPK4","loyalty_member_ref":"mem_McxrCJx-hnxm"},"source":{"version":"3.0.0.Final","connector":"postgresql","name":"odoo","ts_ms":1782674242727,"snapshot":"false","db":"odoo","sequence":"[\"245604512\",\"264078720\"]","ts_us":1782674242727600,"ts_ns":1782674242727600000,"schema":"public","table":"pos_order","txId":6910,"lsn":264078720,"xmin":null},"transaction":null,"op":"u","ts_ms":1782674243017,"ts_us":1782674243017085,"ts_ns":1782674243017085256}', topic: confirmed-orders, partition: 0, offset: 9, timestamp: CreateTime(1782674243290)
```